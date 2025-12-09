import 'package:equatable/equatable.dart';

class PermissionCheckResult extends Equatable {
  final bool isGranted;
  final bool isPermanentlyDenied;

  const PermissionCheckResult({required this.isGranted, required this.isPermanentlyDenied})
    : assert(
        (isGranted && !isPermanentlyDenied) || !isGranted,
        'Permission can not be both granted and permanently denied',
      );

  @override
  List<Object?> get props => [isGranted, isPermanentlyDenied];
}
