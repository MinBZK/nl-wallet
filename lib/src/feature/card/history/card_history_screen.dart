import 'package:flutter/material.dart';

class CardHistoryScreen extends StatelessWidget {
  const CardHistoryScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Card history')),
      body: const Center(
        child: Text('Card history; placeholder.'),
      ),
    );
  }
}
