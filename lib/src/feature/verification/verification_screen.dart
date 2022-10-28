import 'package:flutter/material.dart';

class VerificationScreen extends StatelessWidget {
  const VerificationScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Verification'),
        leading: const BackButton(),
      ),
      body: const Center(
        child: Text('Do you want to share your data with X?'),
      ),
    );
  }
}
