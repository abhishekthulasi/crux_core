import 'package:crux_ui/src/rust/api/simple.dart' as rust_api;
import 'package:crux_ui/src/rust/models.dart';
import 'package:flutter/material.dart';
import 'package:signals/signals_flutter.dart';
import 'chat_state.dart'; // Import your state file

class ChatScreen extends StatefulWidget {
  const ChatScreen({super.key});

  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  final TextEditingController _textController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _initChat();
  }

  Future<void> _initChat() async {
    // For MVP desktop/simulator testing, a local file string is fine.
    // On production mobile, use path_provider to get the documents directory.
    await chatState.initializeDatabase("crux_chat.db");
  }

  String? _editingParentId;
  void _sendMessage() {
    if (_textController.text.trim().isEmpty) return;

    // Call your Rust backend
    chatState.sendMessage(
      _textController.text.trim(),
      parentId: _editingParentId,
    );
    // Reset UI state
    _textController.clear();
    setState(() {
      _editingParentId = null;
    });
  }

  void _startEditing(Turn turn) {
    setState(() {
      // We populate the text box with the old content
      _textController.text = turn.content;
      // The NEW message will share the SAME parent as the old message
      _editingParentId = turn.parentTurnId;
    });
  }

  void _cancelEditing() {
    setState(() {
      _textController.clear();
      _editingParentId = null;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Crux MVP 2')),
      body: Column(
        children: [
          // The Chat List
          Expanded(
            child: Watch((context) {
              final branch = chatState.activeBranch.value;

              if (branch.isEmpty) {
                return const Center(
                  child: Text("Say hello to start the conversation!"),
                );
              }

              return ListView.builder(
                itemCount: branch.length,
                itemBuilder: (context, index) {
                  final turn = branch[index];

                  return BranchingBubble(
                    turn: turn,
                    onEdit: () => _startEditing(turn),
                  );
                },
              );
            }),
          ),

          // The Input Box
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: Row(
              children: [
                // Show a cancel button if we are in edit mode
                if (_editingParentId != null)
                  IconButton(
                    icon: const Icon(Icons.close, color: Colors.red),
                    onPressed: _cancelEditing,
                  ),
                Expanded(
                  child: TextField(
                    controller: _textController,
                    decoration: InputDecoration(
                      hintText: _editingParentId != null
                          ? 'Edit message...'
                          : 'Type a message...',
                      border: const OutlineInputBorder(),
                    ),
                    onSubmitted: (_) => _sendMessage(),
                  ),
                ),
                const SizedBox(width: 8),
                IconButton(
                  icon: const Icon(Icons.send, color: Colors.blue),
                  onPressed: _sendMessage,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

// Add this at the bottom of chat_screen.dart

class BranchingBubble extends StatefulWidget {
  final Turn turn;
  final VoidCallback onEdit;

  const BranchingBubble({super.key, required this.turn, required this.onEdit});

  @override
  State<BranchingBubble> createState() => _BranchingBubbleState();
}

class _BranchingBubbleState extends State<BranchingBubble> {
  List<Turn> _siblings = [];
  int _currentIndex = 0;

  @override
  void initState() {
    super.initState();
    _loadSiblings();
  }

  @override
  void didUpdateWidget(BranchingBubble oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.turn.id != widget.turn.id) {
      _loadSiblings();
    }
  }

  Future<void> _loadSiblings() async {
    final siblings = await rust_api.getSiblings(turnId: widget.turn.id);
    if (!mounted) return;
    setState(() {
      _siblings = siblings;
      _currentIndex = siblings.indexWhere((t) => t.id == widget.turn.id);
    });
  }

  void _cycleBranch(int change) {
    final newIndex = _currentIndex + change;
    if (newIndex >= 0 && newIndex < _siblings.length) {
      // Switch the active conversation to the sibling's timeline!
      chatState.switchBranch(_siblings[newIndex].id);
    }
  }

  @override
  Widget build(BuildContext context) {
    final isUser = widget.turn.role == 'user';
    final hasSiblings = _siblings.length > 1;

    return Align(
      alignment: isUser ? Alignment.centerRight : Alignment.centerLeft,
      child: Column(
        crossAxisAlignment: isUser
            ? CrossAxisAlignment.end
            : CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (isUser)
                IconButton(
                  icon: const Icon(Icons.edit, size: 16, color: Colors.grey),
                  onPressed: widget.onEdit,
                ),
              Container(
                margin: const EdgeInsets.symmetric(vertical: 2, horizontal: 8),
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: isUser ? Colors.blue[100] : Colors.grey[300],
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(widget.turn.content),
              ),
            ],
          ),
          // Branch Cycling Controls
          if (hasSiblings)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16.0),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  InkWell(
                    onTap: () => _cycleBranch(-1),
                    child: const Icon(
                      Icons.chevron_left,
                      size: 18,
                      color: Colors.grey,
                    ),
                  ),
                  Text(
                    '${_currentIndex + 1} / ${_siblings.length}',
                    style: const TextStyle(fontSize: 12, color: Colors.grey),
                  ),
                  InkWell(
                    onTap: () => _cycleBranch(1),
                    child: const Icon(
                      Icons.chevron_right,
                      size: 18,
                      color: Colors.grey,
                    ),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}
