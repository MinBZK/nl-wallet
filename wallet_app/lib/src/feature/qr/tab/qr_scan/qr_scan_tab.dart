import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../../../environment.dart';
import '../../../../domain/model/qr/qr_request.dart';
import '../../../../navigation/wallet_routes.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../common/sheet/explanation_sheet.dart';
import '../../../common/widget/button/text_icon_button.dart';
import '../../../common/widget/centered_loading_indicator.dart';
import '../../../common/widget/loading_indicator.dart';
import '../../../common/widget/utility/check_permission_on_resume.dart';
import '../../../issuance/argument/issuance_screen_argument.dart';
import '../../widget/qr_scanner.dart';
import '../../widget/qr_scanner_frame.dart';
import 'bloc/qr_scan_bloc.dart';

final _scannerKey = Environment.isTest ? ValueKey(DateTime.now()) : GlobalKey();
const _kLandscapePaddingPercent = 0.2;

class QrScanTab extends StatelessWidget {
  const QrScanTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final screenWidth = MediaQuery.of(context).size.width;
    return BlocListener<QrScanBloc, QrScanState>(
      listenWhen: (prev, current) => current is QrScanSuccess,
      listener: (context, state) {
        if (state is QrScanSuccess) _handleScanSuccess(context, state);
      },
      child: ListView(
        padding: EdgeInsets.symmetric(
          vertical: 8,
          horizontal: context.isLandscape ? screenWidth * _kLandscapePaddingPercent : 0,
        ),
        children: [
          BlocListener<QrScanBloc, QrScanState>(
            listener: (context, state) {
              if (state is QrScanSuccess) {
                SemanticsService.announce(context.l10n.qrScanTabCameraScanningQrScannedAnnouncement, TextDirection.ltr);
              }
            },
            child: BlocBuilder<QrScanBloc, QrScanState>(
              builder: (context, state) {
                return switch (state) {
                  QrScanInitial() => _buildInitialState(context),
                  QrScanFailure() => _buildErrorState(context),
                  QrScanNoPermission() => _buildNoPermission(context, state.permanentlyDenied),
                  QrScanScanning() => _buildScanning(context),
                  QrScanSuccess() => _buildSuccessState(context),
                  QrScanLoading() => _buildLoading(),
                };
              },
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: TextIconButton(
              onPressed: () => _showHowToScanSheet(context),
              child: Text(context.l10n.qrScanTabHowToScanCta),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildLoading() {
    return Stack(
      alignment: Alignment.center,
      children: [
        QrScanner(key: _scannerKey),
        Container(
          width: 60,
          height: 60,
          padding: const EdgeInsets.all(16),
          alignment: Alignment.center,
          decoration: const BoxDecoration(shape: BoxShape.circle, color: Colors.white),
          child: const LoadingIndicator(),
        ),
      ],
    );
  }

  Widget _buildInitialState(BuildContext context) {
    return QrScannerFrame(
      child: Container(
        alignment: Alignment.center,
        color: context.colorScheme.secondaryContainer,
        child: const CenteredLoadingIndicator(),
      ),
    );
  }

  Widget _buildErrorState(BuildContext context) {
    return QrScannerFrame(
      child: Container(
        alignment: Alignment.center,
        color: context.colorScheme.secondaryContainer,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.error_outline, color: context.colorScheme.error),
            const SizedBox(height: 8),
            TextButton(
              onPressed: () => context.read<QrScanBloc>().add(const QrScanReset()),
              child: Text(context.l10n.qrScanTabErrorRetryCta),
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
        color: context.colorScheme.secondaryContainer,
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
                child: Text(
                  context.l10n.qrScanTabGrantPermissionCta,
                  textAlign: TextAlign.center,
                ),
              ),
            )
          ],
        ),
      ),
    );
  }

  Widget _buildScanning(BuildContext context) => QrScanner(key: _scannerKey);

  Widget _buildSuccessState(BuildContext context) {
    return QrScannerFrame(
      child: Container(
        alignment: Alignment.center,
        color: context.colorScheme.secondaryContainer,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.qr_code),
            const SizedBox(height: 8),
            TextButton(
              onPressed: () => context.read<QrScanBloc>().add(const QrScanReset()),
              child: Text(context.l10n.qrScanTabContinueCta),
            )
          ],
        ),
      ),
    );
  }

  void _showHowToScanSheet(BuildContext context) {
    ExplanationSheet.show(
      context,
      title: context.l10n.qrScanTabHowToScanSheetTitle,
      description: context.l10n.qrScanTabHowToScanSheetDescription,
      closeButtonText: context.l10n.qrScanTabHowToScanSheetCloseCta,
    );
  }

  void _handleScanSuccess(BuildContext context, QrScanSuccess state) {
    switch (state.request.type) {
      case QrRequestType.disclosure:
        Navigator.restorablePushNamed(
          context,
          WalletRoutes.disclosureRoute,
          arguments: state.request.sessionId,
        );
        break;
      case QrRequestType.issuance:
        Navigator.restorablePushNamed(
          context,
          WalletRoutes.issuanceRoute,
          arguments: IssuanceScreenArgument(sessionId: state.request.sessionId).toMap(),
        );
        break;
      case QrRequestType.sign:
        Navigator.restorablePushNamed(
          context,
          WalletRoutes.signRoute,
          arguments: state.request.sessionId,
        );
        break;
    }
  }
}
