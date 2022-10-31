import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../../wallet_routes.dart';
import '../../../common/widget/bottom_sheet_drag_handle.dart';
import '../../../common/widget/centered_loading_indicator.dart';
import '../../../common/widget/check_permission_on_resume.dart';
import '../../../common/widget/text_arrow_button.dart';
import '../../widget/qr_scanner.dart';
import '../../widget/qr_scanner_frame.dart';
import 'bloc/scan_qr_bloc.dart';

class ScanQrTab extends StatelessWidget {
  const ScanQrTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocListener<ScanQrBloc, ScanQrState>(
      listenWhen: (prev, current) => current is ScanQrSuccess,
      listener: (context, state) => Navigator.restorablePushNamed(context, WalletRoutes.verificationRoute),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const SizedBox(height: 8),
          BlocBuilder<ScanQrBloc, ScanQrState>(
            builder: (context, state) {
              if (state is ScanQrInitial) return _buildInitialState(context);
              if (state is ScanQrError) return _buildErrorState(context);
              if (state is ScanQrNoPermission) return _buildNoPermission(context, state.permanentlyDenied);
              if (state is ScanQrScanning) return QrScanner();
              if (state is ScanQrSuccess) return _buildSuccessState(context);
              throw UnsupportedError('Unknown state: $state');
            },
          ),
          TextArrowButton(
            onPressed: () => _showHowToScanSheet(context),
            child: Text(AppLocalizations.of(context).qrScreenHowToScanButton),
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
              onPressed: () => context.read<ScanQrBloc>().add(const ScanQrReset()),
              child: Text(AppLocalizations.of(context).qrScreenScanTabErrorButton),
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
              onPermissionGranted: () => context.read<ScanQrBloc>().add(const ScanQrCheckPermission()),
              child: TextArrowButton(
                onPressed: () {
                  if (isPermanentlyDenied) {
                    openAppSettings();
                  } else {
                    context.read<ScanQrBloc>().add(const ScanQrCheckPermission());
                  }
                },
                child: Text(AppLocalizations.of(context).qrScreenScanTabGrantPermissionButton),
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
              onPressed: () => context.read<ScanQrBloc>().add(const ScanQrReset()),
              child: Text(AppLocalizations.of(context).qrScreenScanTabContinueButton),
            )
          ],
        ),
      ),
    );
  }

  void _showHowToScanSheet(BuildContext context) {
    showModalBottomSheet<void>(
      context: context,
      builder: (BuildContext context) {
        final locale = AppLocalizations.of(context);
        return Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              const Center(
                child: BottomSheetDragHandle(),
              ),
              const SizedBox(height: 24),
              Text(
                locale.qrScreenHowToScanSheetTitle,
                style: Theme.of(context).textTheme.headline2,
                textAlign: TextAlign.start,
              ),
              const SizedBox(height: 16),
              Text(
                locale.qrScreenHowToScanSheetDescription,
                style: Theme.of(context).textTheme.bodyText1,
              ),
              const SizedBox(height: 16),
              Center(
                child: TextButton(
                  child: Text(locale.qrScreenHowToScanSheetCloseButton),
                  onPressed: () => Navigator.pop(context),
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}
