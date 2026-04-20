import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../common/widget/button/button_content.dart';
import '../../../common/widget/button/tertiary_button.dart';
import '../../../common/widget/utility/check_permissions_on_resume.dart';
import '../bloc/qr_scan_bloc.dart';

class QrScannerNoPermission extends StatelessWidget {
  final bool isPermanentlyDenied;

  const QrScannerNoPermission({required this.isPermanentlyDenied, super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      alignment: Alignment.center,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          SizedBox(height: context.mediaQuery.padding.top),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Text(
              context.l10n.qrScreenPermissionHint,
              textAlign: TextAlign.center,
              style: context.textTheme.bodyLarge,
            ),
          ),
          const Spacer(),
          Icon(
            Icons.camera_alt_outlined,
            color: context.colorScheme.onSurfaceVariant,
          ),
          const SizedBox(height: 8),
          CheckPermissionsOnResume(
            permissions: const [Permission.camera],
            onPermissionGranted: () => context.read<QrScanBloc>().add(const QrScanCheckPermission()),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: TertiaryButton(
                onPressed: () {
                  if (isPermanentlyDenied) {
                    openAppSettings();
                  } else {
                    context.read<QrScanBloc>().add(const QrScanCheckPermission());
                  }
                },
                iconPosition: IconPosition.end,
                text: Text.rich(context.l10n.qrScanTabGrantPermissionCta.toTextSpan(context)),
              ),
            ),
          ),
          const Spacer(),
        ],
      ),
    );
  }
}
