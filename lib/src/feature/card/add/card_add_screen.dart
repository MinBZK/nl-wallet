import 'package:flutter/material.dart';

class CardAddScreen extends StatelessWidget {
  const CardAddScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Card add'),
      ),
      body: const SafeArea(
        child: Center(
          child: Text(
            'Placeholder; card add',
          ),
        ),
      ),
    );
  }
}
