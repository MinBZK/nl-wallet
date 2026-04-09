import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:wallet/src/domain/model/permission/permission_check_result.dart';
import 'package:wallet/src/domain/usecase/permission/impl/check_permission_usecase_impl.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late CheckPermissionUseCaseImpl checkPermissionUseCase;
  const MethodChannel channel = MethodChannel('flutter.baseflow.com/permissions/methods');

  setUp(() {
    checkPermissionUseCase = CheckPermissionUseCaseImpl();
  });

  group('CheckPermissionUseCaseImpl', () {
    test('returns granted when permission is granted', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        if (methodCall.method == 'checkPermissionStatus') {
          return 1; // PermissionStatus.granted
        }
        return null;
      });

      final result = await checkPermissionUseCase.invoke([Permission.camera]);

      expect(result, const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
    });

    test('returns permanently denied when permission is permanently denied', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        if (methodCall.method == 'checkPermissionStatus') {
          return 4; // PermissionStatus.permanentlyDenied
        }
        return null;
      });

      final result = await checkPermissionUseCase.invoke([Permission.camera]);

      expect(result, const PermissionCheckResult(isGranted: false, isPermanentlyDenied: true));
    });

    test('returns denied when permission is denied', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        if (methodCall.method == 'checkPermissionStatus') {
          return 0; // PermissionStatus.denied
        }
        return null;
      });

      final result = await checkPermissionUseCase.invoke([Permission.camera]);

      expect(result, const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
    });

    test('returns granted when all permissions are granted', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        if (methodCall.method == 'checkPermissionStatus') {
          return 1; // PermissionStatus.granted
        }
        return null;
      });

      final result = await checkPermissionUseCase.invoke([Permission.camera, Permission.bluetoothConnect]);

      expect(result, const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
    });

    test('returns denied when any permission is denied', () async {
      final statuses = <int>[
        1, // PermissionStatus.granted (camera - isGranted check)
        0, // PermissionStatus.denied (bluetoothConnect - isGranted check)
        0, // PermissionStatus.denied (bluetoothConnect - isPermanentlyDenied check)
      ];
      int callIndex = 0;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        if (methodCall.method == 'checkPermissionStatus') {
          return statuses[callIndex++];
        }
        return null;
      });

      final result = await checkPermissionUseCase.invoke([Permission.camera, Permission.bluetoothConnect]);

      expect(result, const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
    });

    test('returns permanently denied when any permission is permanently denied', () async {
      final statuses = <int>[
        1, // PermissionStatus.granted (camera - isGranted check)
        4, // PermissionStatus.permanentlyDenied (bluetoothConnect - isGranted check)
        4, // PermissionStatus.permanentlyDenied (bluetoothConnect - isPermanentlyDenied check)
      ];
      int callIndex = 0;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        if (methodCall.method == 'checkPermissionStatus') {
          return statuses[callIndex++];
        }
        return null;
      });

      final result = await checkPermissionUseCase.invoke([Permission.camera, Permission.bluetoothConnect]);

      expect(result, const PermissionCheckResult(isGranted: false, isPermanentlyDenied: true));
    });

    test('returns default result when exception occurs', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        throw Exception('Test error');
      });

      final result = await checkPermissionUseCase.invoke([Permission.camera]);

      expect(result, const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
    });
  });
}
