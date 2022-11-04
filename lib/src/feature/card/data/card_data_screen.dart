import 'package:flutter/material.dart';

class CardDataScreen extends StatelessWidget {
  const CardDataScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Card data')),
      body: const Center(
        child: Text('Card data; placeholder.'),
      ),
    );
  }
}
