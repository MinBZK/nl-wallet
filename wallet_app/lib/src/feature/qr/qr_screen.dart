import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../environment.dart';
import '../../data/service/navigation_service.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/wallet_app_bar.dart';
import 'bloc/qr_bloc.dart';
import 'widget/qr_no_permission.dart';
import 'widget/qr_scanner.dart';

final _scannerKey = Environment.isTest ? ValueKey(DateTime.now()) : GlobalKey();

class QrScreen extends StatelessWidget {
  const QrScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      extendBodyBehindAppBar: true,
      appBar: _buildTransparentAppBar(context),
      body: Column(
        children: [
          Expanded(child: _buildBody(context)),
          const SafeArea(top: false, child: BottomBackButton()),
        ],
      ),
    );
  }

  PreferredSize _buildTransparentAppBar(BuildContext context) {
    final appBar = const WalletAppBar(
      actions: [HelpIconButton()],
      fadeInTitleOnScroll: false,
    );
    return PreferredSize(
      preferredSize: appBar.preferredSize,
      child: Opacity(opacity: 0.9, child: appBar),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocConsumer<QrBloc, QrState>(
      listener: (context, state) {
        _announceState(context, state);
        if (state is QrScanSuccess) _handleScanSuccess(context, state);
        if (state is QrScanFailure) _showInvalidQrDialog(context);
      },
      builder: (context, state) {
        return switch (state) {
          QrScanInitial() => _buildInitialState(),
          QrScanFailure() => _buildQrInvalidState(),
          QrScanNoPermission() => _buildNoPermission(state.permanentlyDenied),
          QrScanScanning() => _buildScanning(),
          QrScanSuccess() => _buildSuccessState(context),
          QrScanLoading() => _buildLoading(context),
        };
      },
    );
  }

  /// Announces the current scanner state when accessibility settings are enabled.
  void _announceState(BuildContext context, QrState state) {
    final String? announcement = switch (state) {
      QrScanScanning() => context.l10n.qrScreenScanningWCAGAnnouncement,
      QrScanLoading() => context.l10n.qrScreenCheckingCodeWCAGAnnouncement,
      QrScanSuccess() => context.l10n.qrScreenScanSuccessfulWCAGAnnouncement,
      _ => null,
    };
    if (announcement != null) {
      SemanticsService.announce(announcement, TextDirection.ltr);
    }
  }

  void _handleScanSuccess(BuildContext context, QrScanSuccess state) {
    final NavigationService navigationService = context.read();
    navigationService.handleNavigationRequest(state.request);
  }

  _buildInitialState() => const SizedBox.shrink();

  _buildQrInvalidState() => QrScanner(key: _scannerKey);

  _buildNoPermission(bool isPermanentlyDenied) => QrNoPermission(isPermanentlyDenied: isPermanentlyDenied);

  _buildScanning() => QrScanner(key: _scannerKey);

  _buildSuccessState(BuildContext context) => Container(
        alignment: Alignment.center,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.qr_code, color: context.colorScheme.onSurfaceVariant),
            const SizedBox(height: 8),
            TextButton(
              onPressed: () => context.read<QrBloc>().add(const QrScanReset()),
              child: Text.rich(context.l10n.qrScanTabContinueCta.toTextSpan(context)),
            ),
          ],
        ),
      );

  _buildLoading(BuildContext context) => Stack(
        alignment: Alignment.center,
        children: [
          Visibility(
            visible: false,
            maintainState: true,
            child: QrScanner(key: _scannerKey),
          ),
          const CenteredLoadingIndicator(),
        ],
      );

  Future<void> _showInvalidQrDialog(BuildContext context) async {
    final qrScanBloc = context.read<QrBloc>();
    await showDialog(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: Text.rich(context.l10n.invalidQrDialogTitle.toTextSpan(context)),
          content: Text.rich(context.l10n.invalidQrDialogDescription.toTextSpan(context)),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: Text.rich(context.l10n.invalidQrDialogCta.toTextSpan(context)),
            ),
          ],
        );
      },
    );
    // Wait until dialog is popped before restarting scanner, to avoid duplicate dialogs.
    qrScanBloc.add(const QrScanReset());
  }
}
