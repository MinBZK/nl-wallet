import 'package:flutter/material.dart';

class CardSummaryScreen extends StatelessWidget {
  const CardSummaryScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Card summary'),
      ),
      body: const SafeArea(
        child: Center(
          child: Text('Placeholder; card summary'),
        ),
      ),
    );
  }
}
