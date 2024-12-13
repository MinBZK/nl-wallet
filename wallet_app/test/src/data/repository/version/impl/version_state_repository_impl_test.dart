import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/version/impl/version_state_repository_impl.dart';
import 'package:wallet/src/domain/model/update/version_state.dart';
import 'package:wallet/src/util/mapper/version/flutter_version_state_mapper.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockTypedWalletCore mockCore;
  late FlutterVersionStateMapper mapper;
  late VersionStateRepositoryImpl versionStateRepository;

  setUp(() {
    mockCore = MockTypedWalletCore();
    mapper = FlutterVersionStateMapper();
    versionStateRepository = VersionStateRepositoryImpl(mockCore, mapper);
  });

  test('verify that VersionStateRepository fetches configuration through wallet_core', () async {
    when(mockCore.observeVersionState()).thenAnswer(
      (_) => Stream.value(const FlutterVersionState_Block()),
    );

    final versionState = await versionStateRepository.observeVersionState().first;
    expect(
      versionState,
      VersionStateBlock(),
    );
  });
}
