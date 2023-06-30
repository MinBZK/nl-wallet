import 'dart:math';

import 'package:flutter/material.dart';

import '../../../wallet_constants.dart';
import '../../pin/widget/pin_field.dart';

class PinFieldDemo extends StatefulWidget {
  const PinFieldDemo({super.key});

  @override
  State<PinFieldDemo> createState() => _PinFieldDemoState();
}

class _PinFieldDemoState extends State<PinFieldDemo> {
  var enteredDigits = 0;
  var fieldState = PinFieldState.idle;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Center(
          child: PinField(
            digits: 6,
            enteredDigits: enteredDigits,
            state: fieldState,
          ),
        ),
        const SizedBox(height: 32),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextButton(
              onPressed: () {
                setState(() {
                  enteredDigits = max(0, enteredDigits - 1);
                });
              },
              child: const Text('-'),
            ),
            TextButton(
              onPressed: () {
                setState(() => enteredDigits = 0);
              },
              child: const Text('clear'),
            ),
            TextButton(
              onPressed: () {
                setState(() {
                  enteredDigits = min(kPinDigits, enteredDigits + 1);
                });
              },
              child: const Text('+'),
            ),
          ],
        ),
        const SizedBox(height: 16),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextButton(
              onPressed: () {
                setState(() => fieldState = PinFieldState.idle);
              },
              child: const Text('IDLE'),
            ),
            TextButton(
              onPressed: () {
                setState(() => fieldState = PinFieldState.loading);
              },
              child: const Text('LOADING'),
            ),
            TextButton(
              onPressed: () {
                setState(() => fieldState = PinFieldState.error);
              },
              child: const Text('ERROR'),
            ),
          ],
        ),
      ],
    );
  }
}
