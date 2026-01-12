abstract class RevocationCodeStore {
  Future<bool> getRevocationCodeSavedFlag();

  Future<void> setRevocationCodeSavedFlag({required bool saved});
}
