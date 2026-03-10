import 'package:crux_ui/chat_screen.dart';
import 'package:crux_ui/src/rust/frb_generated.dart';
import 'package:flutter/material.dart';

void main() async {
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Crux',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(colorScheme: .fromSeed(seedColor: Colors.deepPurple)),
      home: const ChatScreen(),
    );
  }
}
