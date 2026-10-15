import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../common/page/terminal_page.dart';
import '../../../common/widget/button/primary_button.dart';
import '../../../common/widget/button/tertiary_button.dart';
import '../../../common/widget/page_illustration.dart';
import '../../../common/widget/utility/check_permissions_on_resume.dart';
import '../bloc/qr_scan_bloc.dart';

class QrScannerNoPermission extends StatelessWidget {
  final bool isPermanentlyDenied;

  const QrScannerNoPermission({required this.isPermanentlyDenied, super.key});

  @override
  Widget build(BuildContext context) {
    return CheckPermissionsOnResume(
      permissions: const [Permission.camera],
      onPermissionGranted: () => context.read<QrScanBloc>().add(const QrScanCheckPermission()),
      child: TerminalPage(
        title: context.l10n.qrScreenPermissionTitle,
        description: context.l10n.qrScreenPermissionHint,
        illustration: const PageIllustration(asset: WalletAssets.svg_camera_permission),
        primaryButton: PrimaryButton(
          onPressed: () {
            if (isPermanentlyDenied) {
              openAppSettings();
            } else {
              context.read<QrScanBloc>().add(const QrScanCheckPermission());
            }
          },
          icon: const Icon(Icons.camera_alt),
          text: Text.rich(context.l10n.qrScanTabGrantPermissionCta.toTextSpan(context)),
        ),
        secondaryButton: TertiaryButton(
          onPressed: () => Navigator.maybePop(context),
          icon: const Icon(Icons.arrow_back),
          text: Text.rich(context.l10n.generalBottomBackCta.toTextSpan(context)),
        ),
      ),
    );
  }
}
