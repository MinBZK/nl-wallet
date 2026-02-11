import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/revocation/bloc/revocation_code_bloc.dart';
import 'package:wallet/src/feature/revocation/revocation_code_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

class MockRevocationCodeBloc extends MockBloc<RevocationCodeEvent, RevocationCodeState> implements RevocationCodeBloc {}

const sampleRevocationCode = 'AB12CD34EF56GH78IJ';

void main() {
  group('RevocationCodeScreen', () {
    testGoldens('RevocationCodeInitial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RevocationCodeScreen().withState<RevocationCodeBloc, RevocationCodeState>(
          MockRevocationCodeBloc(),
          const RevocationCodeInitial(),
        ),
      );
      await screenMatchesGolden('revocation_code_initial.light');
    });

    testGoldens('ltc70 RevocationCodeLoadSuccess', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RevocationCodeScreen().withState<RevocationCodeBloc, RevocationCodeState>(
          MockRevocationCodeBloc(),
          const RevocationCodeLoadSuccess(sampleRevocationCode),
        ),
      );
      await screenMatchesGolden('revocation_code_load_success.light');
    });

    testGoldens('ltc70 RevocationCodeLoadSuccess - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RevocationCodeScreen().withState<RevocationCodeBloc, RevocationCodeState>(
          MockRevocationCodeBloc(),
          const RevocationCodeLoadSuccess(sampleRevocationCode),
        ),
        brightness: .dark,
      );
      await screenMatchesGolden('revocation_code_load_success.dark');
    });

    testGoldens('ltc70 RevocationCodeLoadSuccess - 2x - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RevocationCodeScreen().withState<RevocationCodeBloc, RevocationCodeState>(
          MockRevocationCodeBloc(),
          const RevocationCodeLoadSuccess(sampleRevocationCode),
        ),
        textScaleSize: 2,
        brightness: .dark,
      );
      await screenMatchesGolden('revocation_code_load_success.dark.scaled');
    });

    testGoldens('ltc70 RevocationCodeLoadSuccess - landscape - 2x', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RevocationCodeScreen().withState<RevocationCodeBloc, RevocationCodeState>(
          MockRevocationCodeBloc(),
          const RevocationCodeLoadSuccess(sampleRevocationCode),
        ),
        surfaceSize: iphoneXSizeLandscape,
        textScaleSize: 2,
      );
      await screenMatchesGolden('revocation_code_load_success.light.landscape.scaled');
    });
  });
}
