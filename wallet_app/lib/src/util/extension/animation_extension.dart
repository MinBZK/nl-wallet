import 'package:flutter/material.dart';

extension AnimationExtension<T> on Animation<T> {
  void addOnCompleteListener(VoidCallback onComplete) {
    statusListener(status) {
      if (status == AnimationStatus.completed) {
        onComplete();
        removeStatusListener(statusListener);
      }
    }

    addStatusListener(statusListener);
  }
}
