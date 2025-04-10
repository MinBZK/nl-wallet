// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.9.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:freezed_annotation/freezed_annotation.dart' hide protected;
part 'version_state.freezed.dart';

@freezed
sealed class FlutterVersionState with _$FlutterVersionState {
  const FlutterVersionState._();

  const factory FlutterVersionState.ok() = FlutterVersionState_Ok;
  const factory FlutterVersionState.notify() = FlutterVersionState_Notify;
  const factory FlutterVersionState.recommend() = FlutterVersionState_Recommend;
  const factory FlutterVersionState.warn({
    required BigInt expiresInSeconds,
  }) = FlutterVersionState_Warn;
  const factory FlutterVersionState.block() = FlutterVersionState_Block;
}
