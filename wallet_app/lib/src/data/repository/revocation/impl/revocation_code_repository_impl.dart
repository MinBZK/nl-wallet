import 'package:wallet_core/core.dart';

import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../store/revocation_code_store.dart';
import '../revocation_code_repository.dart';

class RevocationRepositoryImpl extends RevocationRepository {
  final TypedWalletCore _core;
  final RevocationCodeStore _store;

  RevocationRepositoryImpl(this._core, this._store);

  @override
  Future<bool> getRevocationCodeSaved() {
    return _store.getRevocationCodeSavedFlag();
  }

  @override
  Future<void> setRevocationCodeSaved({required bool saved}) async {
    await _store.setRevocationCodeSavedFlag(saved: saved);
  }

  @override
  Future<String> getRegistrationRevocationCode() => _core.getRegistrationRevocationCode();

  @override
  Future<String> getRevocationCode(String pin) async {
    final result = await _core.getRevocationCode(pin);
    switch (result) {
      case RevocationCodeResult_Ok():
        return result.revocationCode;
      case RevocationCodeResult_InstructionError():
        throw result.error;
    }
  }
}
