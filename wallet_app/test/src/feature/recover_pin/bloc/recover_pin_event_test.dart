import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/recover_pin/bloc/recover_pin_bloc.dart';

void main() {
  group('RecoverPinEvent equality', () {
    test('RecoverPinLoginWithDigidClicked equality', () {
      expect(
        const RecoverPinLoginWithDigidClicked(),
        equals(const RecoverPinLoginWithDigidClicked()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RecoverPinContinuePinRecovery equality', () {
      expect(
        const RecoverPinContinuePinRecovery('url1'),
        equals(const RecoverPinContinuePinRecovery('url1')),
        reason: 'Instances with same authUrl should be equal',
      );
      expect(
        const RecoverPinContinuePinRecovery('url1'),
        isNot(equals(const RecoverPinContinuePinRecovery('url2'))),
        reason: 'Instances with different authUrl should not be equal',
      );
    });

    test('RecoverPinDigitPressed equality', () {
      expect(
        const RecoverPinDigitPressed(1),
        equals(const RecoverPinDigitPressed(1)),
        reason: 'Instances with same digit should be equal',
      );
      expect(
        const RecoverPinDigitPressed(1),
        isNot(equals(const RecoverPinDigitPressed(2))),
        reason: 'Instances with different digit should not be equal',
      );
    });

    test('RecoverPinBackspacePressed equality', () {
      expect(
        RecoverPinBackspacePressed(),
        equals(RecoverPinBackspacePressed()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RecoverPinClearPressed equality', () {
      expect(
        RecoverPinClearPressed(),
        equals(RecoverPinClearPressed()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RecoverPinNewPinConfirmed equality', () {
      expect(
        const RecoverPinNewPinConfirmed(pin: '123456', authUrl: 'url'),
        equals(const RecoverPinNewPinConfirmed(pin: '123456', authUrl: 'url')),
        reason: 'Same pin and authUrl should be equal',
      );
      expect(
        const RecoverPinNewPinConfirmed(pin: '123456', authUrl: 'url'),
        isNot(equals(const RecoverPinNewPinConfirmed(pin: '654321', authUrl: 'url'))),
        reason: 'Different pin should not be equal',
      );
      expect(
        const RecoverPinNewPinConfirmed(pin: '123456', authUrl: 'url1'),
        isNot(equals(const RecoverPinNewPinConfirmed(pin: '123456', authUrl: 'url2'))),
        reason: 'Different authUrl should not be equal',
      );
    });

    test('RecoverPinLaunchDigidUrlFailed equality', () {
      final err1 = const GenericError('fail1', sourceError: 'fail1');
      final err2 = const GenericError('fail2', sourceError: 'fail2');
      expect(
        RecoverPinLaunchDigidUrlFailed(error: err1),
        equals(RecoverPinLaunchDigidUrlFailed(error: err1)),
      );
      expect(
        RecoverPinLaunchDigidUrlFailed(error: err1),
        isNot(equals(RecoverPinLaunchDigidUrlFailed(error: err2))),
      );
    });

    test('RecoverPinLoginWithDigidFailed equality', () {
      final err1 = const GenericError('fail1', sourceError: 'fail1');
      final err2 = const GenericError('fail2', sourceError: 'fail2');
      expect(
        RecoverPinLoginWithDigidFailed(error: err1, cancelledByUser: false),
        equals(RecoverPinLoginWithDigidFailed(error: err1, cancelledByUser: false)),
        reason: 'Same error and flag should be equal',
      );
      expect(
        RecoverPinLoginWithDigidFailed(error: err1, cancelledByUser: true),
        isNot(equals(RecoverPinLoginWithDigidFailed(error: err1, cancelledByUser: false))),
        reason: 'Same error, different flag should not be equal',
      );
      expect(
        RecoverPinLoginWithDigidFailed(error: err1, cancelledByUser: false),
        isNot(equals(RecoverPinLoginWithDigidFailed(error: err2, cancelledByUser: false))),
        reason: 'Different error, same flag should not be equal',
      );
    });

    test('RecoverPinRetryPressed equality', () {
      expect(
        const RecoverPinRetryPressed(),
        equals(const RecoverPinRetryPressed()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RecoverPinStopPressed equality', () {
      expect(
        const RecoverPinStopPressed(),
        equals(const RecoverPinStopPressed()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RecoverPinBackPressed equality', () {
      expect(
        const RecoverPinBackPressed(),
        equals(const RecoverPinBackPressed()),
        reason: 'Instances with no fields should be equal',
      );
    });
  });
}
