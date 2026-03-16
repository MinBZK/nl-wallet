import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:wallet/src/domain/model/permission/permission_check_result.dart';
import 'package:wallet/src/domain/usecase/permission/impl/request_permission_usecase_impl.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late RequestPermissionUseCaseImpl requestPermissionUseCase;
  const MethodChannel channel = MethodChannel('flutter.baseflow.com/permissions/methods');

  setUp(() {
    requestPermissionUseCase = RequestPermissionUseCaseImpl();
  });

  group('RequestPermissionUseCaseImpl', () {
    test('returns granted when permission is requested and granted', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        if (methodCall.method == 'requestPermissions') {
          return {
            Permission.camera.value: 1, // PermissionStatus.granted
          };
        }
        return null;
      });

      final result = await requestPermissionUseCase.invoke(Permission.camera);

      expect(result, const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
    });

    test('returns permanently denied when permission is requested and permanently denied', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        if (methodCall.method == 'requestPermissions') {
          return {
            Permission.camera.value: 4, // PermissionStatus.permanentlyDenied
          };
        }
        if (methodCall.method == 'checkPermissionStatus') {
          return 4; // PermissionStatus.permanentlyDenied
        }
        return null;
      });

      final result = await requestPermissionUseCase.invoke(Permission.camera);

      expect(result, const PermissionCheckResult(isGranted: false, isPermanentlyDenied: true));
    });

    test('returns denied when permission is requested and denied', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        if (methodCall.method == 'requestPermissions') {
          return {
            Permission.camera.value: 0, // PermissionStatus.denied
          };
        }
        if (methodCall.method == 'checkPermissionStatus') {
          return 0; // PermissionStatus.denied
        }
        return null;
      });

      final result = await requestPermissionUseCase.invoke(Permission.camera);

      expect(result, const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
    });

    test('returns default result when exception occurs', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
        MethodCall methodCall,
      ) async {
        throw Exception('Test error');
      });

      final result = await requestPermissionUseCase.invoke(Permission.camera);

      expect(result, const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false));
    });
  });
}
