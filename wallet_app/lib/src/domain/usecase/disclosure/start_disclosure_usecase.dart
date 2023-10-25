import '../../../data/repository/disclosure/disclosure_repository.dart';

export '../../../data/repository/disclosure/disclosure_repository.dart';

abstract class StartDisclosureUseCase {
  Stream<StartDisclosureResult> invoke(Uri disclosureUri);
}
