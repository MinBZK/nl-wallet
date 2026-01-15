abstract class RevocationRepository {
  Future<void> setRevocationCodeSaved({required bool saved});

  Future<bool> getRevocationCodeSaved();

  Future<String> getRegistrationRevocationCode();

  Future<String> getRevocationCode(String pin);
}
