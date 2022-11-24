import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../../domain/model/qr/qr_request.dart';
import '../../../../wallet_routes.dart';
import '../../../common/widget/centered_loading_indicator.dart';
import '../../../common/widget/check_permission_on_resume.dart';
import '../../../common/widget/explanation_sheet.dart';
import '../../../common/widget/text_icon_button.dart';
import '../../widget/qr_scanner.dart';
import '../../widget/qr_scanner_frame.dart';
import 'bloc/qr_scan_bloc.dart';

class QrScanTab extends StatelessWidget {
  const QrScanTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocListener<QrScanBloc, QrScanState>(
      listenWhen: (prev, current) => current is QrScanSuccess,
      listener: (context, state) {
        if (state is QrScanSuccess) _handleScanSuccess(context, state);
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const SizedBox(height: 8),
          BlocBuilder<QrScanBloc, QrScanState>(
            builder: (context, state) {
              if (state is QrScanInitial) return _buildInitialState(context);
              if (state is QrScanFailure) return _buildErrorState(context);
              if (state is QrScanNoPermission) return _buildNoPermission(context, state.permanentlyDenied);
              if (state is QrScanScanning) return QrScanner();
              if (state is QrScanSuccess) return _buildSuccessState(context);
              throw UnsupportedError('Unknown state: $state');
            },
          ),
          TextIconButton(
            onPressed: () => _showHowToScanSheet(context),
            child: Text(AppLocalizations.of(context).qrScanTabHowToScanCta),
          ),
        ],
      ),
    );
  }

  Widget _buildInitialState(BuildContext context) {
    return QrScannerFrame(
      child: Container(
        alignment: Alignment.center,
        color: Theme.of(context).colorScheme.secondaryContainer,
        child: const CenteredLoadingIndicator(),
      ),
    );
  }

  Widget _buildErrorState(BuildContext context) {
    return QrScannerFrame(
      child: Container(
        alignment: Alignment.center,
        color: Theme.of(context).colorScheme.secondaryContainer,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.error_outline, color: Theme.of(context).errorColor),
            const SizedBox(height: 8),
            TextButton(
              onPressed: () => context.read<QrScanBloc>().add(const QrScanReset()),
              child: Text(AppLocalizations.of(context).qrScanTabErrorRetryCta),
            )
          ],
        ),
      ),
    );
  }

  Widget _buildNoPermission(BuildContext context, bool isPermanentlyDenied) {
    return QrScannerFrame(
      child: Container(
        alignment: Alignment.center,
        color: Theme.of(context).colorScheme.secondaryContainer,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.camera_alt),
            const SizedBox(height: 8),
            CheckPermissionOnResume(
              permission: Permission.camera,
              onPermissionGranted: () => context.read<QrScanBloc>().add(const QrScanCheckPermission()),
              child: TextIconButton(
                onPressed: () {
                  if (isPermanentlyDenied) {
                    openAppSettings();
                  } else {
                    context.read<QrScanBloc>().add(const QrScanCheckPermission());
                  }
                },
                child: Text(AppLocalizations.of(context).qrScanTabGrantPermissionCta),
              ),
            )
          ],
        ),
      ),
    );
  }

  Widget _buildSuccessState(BuildContext context) {
    return QrScannerFrame(
      child: Container(
        alignment: Alignment.center,
        color: Theme.of(context).colorScheme.secondaryContainer,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.qr_code),
            const SizedBox(height: 8),
            TextButton(
              onPressed: () => context.read<QrScanBloc>().add(const QrScanReset()),
              child: Text(AppLocalizations.of(context).qrScanTabContinueCta),
            )
          ],
        ),
      ),
    );
  }

  void _showHowToScanSheet(BuildContext context) {
    final locale = AppLocalizations.of(context);
    ExplanationSheet.show(
      context,
      title: locale.qrScanTabHowToScanSheetTitle,
      description: locale.qrScanTabHowToScanSheetDescription,
      closeButtonText: locale.qrScanTabHowToScanSheetCloseCta,
    );
  }

  void _handleScanSuccess(BuildContext context, QrScanSuccess state) {
    switch (state.request.type) {
      case QrRequestType.verification:
        Navigator.restorablePushNamed(
          context,
          WalletRoutes.verificationRoute,
          arguments: state.request.sessionId,
        );
        break;
      case QrRequestType.issuance:
        Navigator.restorablePushNamed(
          context,
          WalletRoutes.issuanceRoute,
          arguments: state.request.sessionId,
        );
        break;
    }
  }
}
