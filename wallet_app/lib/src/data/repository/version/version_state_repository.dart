import '../../../domain/model/update/version_state.dart';

abstract class VersionStateRepository {
  Stream<VersionState> observeVersionState();
}
