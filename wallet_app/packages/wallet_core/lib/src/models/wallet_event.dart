// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.8.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'attestation.dart';
import 'disclosure.dart';
import 'localize.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:freezed_annotation/freezed_annotation.dart' hide protected;
part 'wallet_event.freezed.dart';

@freezed
sealed class WalletEvent with _$WalletEvent {
  const WalletEvent._();

  const factory WalletEvent.disclosure({
    required String dateTime,
    required Organization relyingParty,
    required List<LocalizedString> purpose,
    List<Attestation>? requestedAttestations,
    required RequestPolicy requestPolicy,
    required DisclosureStatus status,
    required DisclosureType typ,
  }) = WalletEvent_Disclosure;
  const factory WalletEvent.issuance({
    required String dateTime,
    required Attestation attestation,
  }) = WalletEvent_Issuance;
}
