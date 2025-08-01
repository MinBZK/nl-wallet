import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/renew_pid/bloc/renew_pid_bloc.dart';

void main() {
  group('RenewPidEvent equality', () {
    test('RenewPidLoginWithDigidClicked equality', () {
      expect(
        const RenewPidLoginWithDigidClicked(),
        equals(const RenewPidLoginWithDigidClicked()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RenewPidContinuePidRenewal equality', () {
      expect(
        const RenewPidContinuePidRenewal('url1'),
        equals(const RenewPidContinuePidRenewal('url1')),
        reason: 'Instances with same authUrl should be equal',
      );
      expect(
        const RenewPidContinuePidRenewal('url1'),
        isNot(equals(const RenewPidContinuePidRenewal('url2'))),
        reason: 'Instances with different authUrl should not be equal',
      );
    });

    test('RenewPidAttributesRejected equality', () {
      expect(
        const RenewPidAttributesRejected(),
        equals(const RenewPidAttributesRejected()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RenewPidAttributesConfirmed equality', () {
      final attr1 = DataAttribute.untranslated(key: 'a', label: '', value: const StringValue('a'));
      final attr2 = DataAttribute.untranslated(key: 'b', label: '', value: const StringValue('b'));
      expect(
        RenewPidAttributesConfirmed([attr1, attr2]),
        equals(RenewPidAttributesConfirmed([attr1, attr2])),
        reason: 'Same attribute list should be equal',
      );
      expect(
        RenewPidAttributesConfirmed([attr1]),
        isNot(equals(RenewPidAttributesConfirmed([attr2]))),
        reason: 'Different attribute lists should not be equal',
      );
    });

    test('RenewPidPinConfirmed equality', () {
      expect(
        RenewPidPinConfirmed(),
        equals(RenewPidPinConfirmed()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RenewPidPinConfirmationFailed equality', () {
      final err1 = const GenericError('fail1', sourceError: 'fail1');
      final err2 = const GenericError('fail2', sourceError: 'fail2');
      expect(
        RenewPidPinConfirmationFailed(error: err1),
        equals(RenewPidPinConfirmationFailed(error: err1)),
        reason: 'Same error object should be equal',
      );
      expect(
        RenewPidPinConfirmationFailed(error: err1),
        isNot(equals(RenewPidPinConfirmationFailed(error: err2))),
        reason: 'Different error objects should not be equal',
      );
    });

    test('RenewPidLaunchDigidUrlFailed equality', () {
      final err1 = const GenericError('fail1', sourceError: 'fail1');
      final err2 = const GenericError('fail2', sourceError: 'fail2');
      expect(
        RenewPidLaunchDigidUrlFailed(error: err1),
        equals(RenewPidLaunchDigidUrlFailed(error: err1)),
      );
      expect(
        RenewPidLaunchDigidUrlFailed(error: err1),
        isNot(equals(RenewPidLaunchDigidUrlFailed(error: err2))),
      );
    });

    test('RenewPidLoginWithDigidFailed equality', () {
      final err1 = const GenericError('fail1', sourceError: 'fail1');
      final err2 = const GenericError('fail2', sourceError: 'fail2');
      expect(
        RenewPidLoginWithDigidFailed(error: err1, cancelledByUser: false),
        equals(RenewPidLoginWithDigidFailed(error: err1, cancelledByUser: false)),
        reason: 'Same error and flag should be equal',
      );
      expect(
        RenewPidLoginWithDigidFailed(error: err1, cancelledByUser: true),
        isNot(equals(RenewPidLoginWithDigidFailed(error: err1, cancelledByUser: false))),
        reason: 'Same error, different flag should not be equal',
      );
      expect(
        RenewPidLoginWithDigidFailed(error: err1, cancelledByUser: false),
        isNot(equals(RenewPidLoginWithDigidFailed(error: err2, cancelledByUser: false))),
        reason: 'Different error, same flag should not be equal',
      );
    });

    test('RenewPidRetryPressed equality', () {
      expect(
        const RenewPidRetryPressed(),
        equals(const RenewPidRetryPressed()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RenewPidStopPressed equality', () {
      expect(
        const RenewPidStopPressed(),
        equals(const RenewPidStopPressed()),
        reason: 'Instances with no fields should be equal',
      );
    });

    test('RenewPidBackPressed equality', () {
      expect(
        const RenewPidBackPressed(),
        equals(const RenewPidBackPressed()),
        reason: 'Instances with no fields should be equal',
      );
    });
  });
}
