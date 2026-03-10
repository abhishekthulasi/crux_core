import 'package:crux_ui/src/rust/api/simple.dart' as rust_api;
import 'package:crux_ui/src/rust/models.dart';
import 'package:signals/signals.dart';

class ChatState {
  // The active branch of the conversation
  final Signal<List<Turn>> activeBranch = signal([]);

  // Keep track of the ID of the last message in the current view
  final Signal<String?> currentLeafId = signal(null);

  Future<void> initializeDatabase(String path) async {
    await rust_api.initDb(dbPath: path);
  }

  // Add the optional parentId parameter
  Future<void> sendMessage(String text, {String? parentId}) async {
    // Use the provided parentId if it exists, otherwise use the current leaf
    final targetParent = parentId ?? currentLeafId.value;

    final updatedBranch = await rust_api.sendMessage(
      parentId: targetParent,
      content: text,
    );

    activeBranch.value = updatedBranch;

    if (updatedBranch.isNotEmpty) {
      currentLeafId.value = updatedBranch.last.id;
    }
  }

  Future<void> loadBranch(String leafNodeId) async {
    final branch = await rust_api.fetchBranch(activeTurnId: leafNodeId);
    activeBranch.value = branch;
    currentLeafId.value = leafNodeId;
  }

  Future<void> switchBranch(String turnId) async {
    final newBranch = await rust_api.switchBranch(turnId: turnId);

    activeBranch.value = newBranch;
    if (newBranch.isNotEmpty) {
      currentLeafId.value = newBranch.last.id;
    }
  }
}

final chatState = ChatState();
