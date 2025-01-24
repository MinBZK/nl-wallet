// coverage:ignore-file
import 'dart:ui';

import 'package:flutter/material.dart';

import '../widget/digid_confirm_buttons.dart';
import '../widget/digid_sign_in_with_header.dart';
import '../widget/digid_sign_in_with_organization.dart';

class DigidLoadingPage extends StatelessWidget {
  final Duration mockDelay;

  const DigidLoadingPage({required this.mockDelay, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Stack(
          children: [
            _buildBackground(),
            BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
              child: Container(
                color: Colors.grey.withValues(alpha: 0.1),
                alignment: Alignment.center,
              ),
            ),
            Center(child: _buildProgressIndicator()),
          ],
        ),
      ),
    );
  }

  Widget _buildBackground() {
    return const Column(
      mainAxisSize: MainAxisSize.max,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Align(
          alignment: Alignment.centerRight,
          child: Icon(Icons.close),
        ),
        DigidSignInWithHeader(),
        Spacer(),
        Center(child: DigidSignInWithOrganization()),
        Spacer(),
        DigidConfirmButtons(),
      ],
    );
  }

  Widget _buildProgressIndicator() {
    const color = Colors.green;
    return TweenAnimationBuilder<double>(
      builder: (context, progress, child) {
        return Stack(
          alignment: Alignment.center,
          children: [
            Container(
              width: 90,
              height: 90,
              alignment: Alignment.center,
              decoration: const BoxDecoration(shape: BoxShape.circle, color: Colors.white),
              child: SizedBox(
                height: 60,
                width: 60,
                child: CircularProgressIndicator(
                  value: progress,
                  color: color,
                  strokeWidth: 6,
                ),
              ),
            ),
            child!,
          ],
        );
      },
      duration: mockDelay,
      tween: Tween<double>(begin: 0, end: 1),
      child: const Icon(
        Icons.check,
        color: color,
        size: 32,
      ),
    );
  }
}
