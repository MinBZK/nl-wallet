abstract class HasPreviouslyInteractedWithOrganizationUseCase {
  /// True if the user has previously successfully (!) interacted with the organization
  /// associated to the provided [organizationId].
  Future<bool> invoke(String organizationId);
}
