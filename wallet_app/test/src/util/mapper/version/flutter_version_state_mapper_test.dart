import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/update/version_state.dart';
import 'package:wallet/src/util/mapper/version/flutter_version_state_mapper.dart';
import 'package:wallet_core/core.dart';

void main() {
  late FlutterVersionStateMapper mapper;

  setUp(() {
    mapper = FlutterVersionStateMapper();
  });

  test('FlutterVersionStateMapper maps to the expected values', () {
    const warningSeconds = 980;

    expect(mapper.map(const FlutterVersionState.ok()), VersionStateOk());
    expect(mapper.map(const FlutterVersionState.notify()), VersionStateNotify());
    expect(mapper.map(const FlutterVersionState.recommend()), VersionStateRecommend());
    expect(
      mapper.map(const FlutterVersionState.warn(expiresInSeconds: warningSeconds)),
      VersionStateWarn(timeUntilBlocked: const Duration(seconds: warningSeconds)),
    );
    expect(mapper.map(const FlutterVersionState.block()), VersionStateBlock());
  });
}
