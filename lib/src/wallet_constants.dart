import 'package:flutter/foundation.dart';

// Animation
const kDefaultAnimationDuration = Duration(milliseconds: 300);

// Security
const kPinDigits = 6;
const kMaxUnlockAttempts = 3;
const kMockPin = '123456';
const kBackgroundLockTimeout = Duration(minutes: kDebugMode ? 10 : 5);
const kIdleLockTimeout = Duration(minutes: kDebugMode ? 50 : 20);

// Mocking
const kDefaultMockDelay = Duration(milliseconds: 500);
const kDefaultDigidMockDelay = Duration(seconds: 2);
