import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../../domain/model/result/application_error.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../theme/light_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/helper/dialog_helper.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_constants.dart';
import '../../common/dialog/qr_code_dialog.dart';
import '../../common/page/generic_loading_page.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/icon/back_icon_button.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/utility/check_permissions_on_resume.dart';
import '../../common/widget/utility/scroll_offset_provider.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../common/widget/wallet_scrollbar.dart';
import '../../disclosure/argument/disclosure_screen_argument.dart';
import '../../error/error_button_builder.dart';
import '../../error/error_page.dart';
import 'bloc/qr_present_bloc.dart';

class QrPresentScreen extends StatelessWidget {
  const QrPresentScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final state = context.watch<QrPresentBloc>().state;
    final content = switch (state) {
      QrPresentInitial() => _buildInitial(context),
      QrPresentServerStarted(:final qrContents) => _buildServerStarted(context, qrContents),
      QrPresentConnecting() => _buildConnecting(context),
      QrPresentConnected() => _buildConnected(context),
      QrPresentConnectionFailed() => _buildConnectionFailed(context),
      QrPresentError(:final error) => _buildError(context, error),
    };
    return ScrollOffsetProvider(
      debugLabel: 'qr_present_screen',
      child: Scaffold(
        appBar: WalletAppBar(
          fadeInTitleOnScroll: _fadeInTitleOnScroll(state),
          title: Text(_resolveTitle(context, state)),
          actions: const [HelpIconButton()],
          automaticallyImplyLeading: false,
          leading: _leadingButton(state),
        ),
        body: BlocListener<QrPresentBloc, QrPresentState>(
          listener: (context, state) {
            DialogHelper.dismissOpenDialogs(context);
            final navigateToDisclosure = (state as QrPresentConnected).deviceRequestReceived;
            if (navigateToDisclosure) _navigateToDisclosure(context);
          },
          listenWhen: (prev, current) => current is QrPresentConnected,
          child: CheckPermissionsOnResume(
            onPermissionDenied: () => context.read<QrPresentBloc>().add(const QrPresentPermissionDenied()),
            permissions: Platform.isAndroid ? kAndroidBlePermissions : kIosBlePermissions,
            child: SafeArea(child: content),
          ),
        ),
      ),
    );
  }

  void _navigateToDisclosure(BuildContext context) {
    Navigator.pushReplacementNamed(
      context,
      WalletRoutes.disclosureRoute,
      arguments: const DisclosureScreenArgument(type: .closeProximity()),
    );
  }

  bool _fadeInTitleOnScroll(QrPresentState state) => switch (state) {
    QrPresentInitial() => false,
    QrPresentServerStarted() => false,
    QrPresentConnecting() => true,
    QrPresentConnected() => true,
    QrPresentConnectionFailed() => true,
    QrPresentError() => true,
  };

  Widget? _leadingButton(QrPresentState state) {
    return switch (state) {
      QrPresentInitial() => const BackIconButton(),
      QrPresentServerStarted() => const BackIconButton(),
      QrPresentConnecting() => null,
      QrPresentConnected() => null,
      QrPresentConnectionFailed() => null,
      QrPresentError() => const BackIconButton(),
    };
  }

  Widget _buildInitial(BuildContext context) {
    return const Column(
      children: [
        Expanded(child: CenteredLoadingIndicator()),
        BottomBackButton(),
      ],
    );
  }

  String _resolveTitle(BuildContext context, QrPresentState state) {
    return switch (state) {
      QrPresentInitial() => context.l10n.qrPresentScreenTitle,
      QrPresentServerStarted() => context.l10n.qrPresentScreenTitle,
      QrPresentConnecting() => '',
      QrPresentConnected() => '',
      QrPresentConnectionFailed() => context.l10n.qrPresentScreenConnectionFailedPageTitle,
      QrPresentError(:final error) => _buildError(context, error).headline,
    };
  }

  Widget _buildServerStarted(BuildContext context, String qrContents) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: ConstrainedBox(
              constraints: BoxConstraints(minHeight: constraints.maxHeight),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Column(
                    children: [
                      const SizedBox(height: 12),
                      ListButton(
                        text: Text(context.l10n.qrPresentScreenCenterQrCodeCta),
                        onPressed: () => QrCodeDialog.show(
                          context,
                          title: context.l10n.qrPresentScreenDialogTitle,
                          data: qrContents,
                        ),
                      ),
                      const SizedBox(height: 16),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
                        child: QrImageView(
                          padding: EdgeInsets.zero,
                          backgroundColor: LightWalletTheme.colorScheme.surface,
                          dataModuleStyle: const QrDataModuleStyle(
                            color: Colors.black,
                            dataModuleShape: QrDataModuleShape.square,
                          ),
                          data: qrContents,
                          embeddedImage: const AssetImage(WalletAssets.logo_wallet_qr),
                          embeddedImageEmitsError: true,
                          errorCorrectionLevel: QrErrorCorrectLevel.Q,
                          embeddedImageStyle: const QrEmbeddedImageStyle(size: Size(64, 64)),
                        ),
                      ),
                      const SizedBox(height: 16),
                      ListButton(
                        text: Text(context.l10n.qrPresentScreenReportIssueCta),
                        onPressed: () => PlaceholderScreen.showGeneric(context),
                      ),
                    ],
                  ),
                  const BottomBackButton(),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  ErrorPage _buildError(BuildContext context, ApplicationError error) {
    return ErrorPage.fromError(
      context,
      error,
      onPrimaryActionPressed: () => Navigator.pop(context),
      style: .close,
    );
  }

  Widget _buildConnecting(BuildContext context) {
    return GenericLoadingPage(
      title: context.l10n.qrPresentScreenConnectingTitle,
      description: '',
      cancelCta: context.l10n.generalStop,
      onCancel: () => Navigator.pop(context),
    );
  }

  Widget _buildConnected(BuildContext context) {
    return GenericLoadingPage(
      title: context.l10n.qrPresentScreenConnectedTitle,
      description: '',
      cancelCta: context.l10n.generalStop,
      onCancel: () => Navigator.pop(context),
    );
  }

  Widget _buildConnectionFailed(BuildContext context) {
    return ErrorPage(
      headline: context.l10n.qrPresentScreenConnectionFailedPageTitle,
      description: context.l10n.qrPresentScreenConnectionFailedPageDescription,
      illustration: WalletAssets.svg_error_bluetooth,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
        context,
        .retry,
        onPressed: () => context.read<QrPresentBloc>().add(const QrPresentStartRequested()),
      ),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
    );
  }
}
