import 'dart:async';

import '../../../../data/repository/pid/pid_repository.dart';
import '../../../model/result/result.dart';
import '../continue_pid_issuance_usecase.dart';

class ContinuePidIssuanceUseCaseImpl extends ContinuePidIssuanceUseCase {
  final PidRepository _pidRepository;

  ContinuePidIssuanceUseCaseImpl(this._pidRepository);

  @override
  Future<Result<PreviewAttributes>> invoke(String uri) async {
    return tryCatch(
      () async => _pidRepository.continuePidIssuance(uri),
      'Failed to continue pid issuance',
    );
  }
}
