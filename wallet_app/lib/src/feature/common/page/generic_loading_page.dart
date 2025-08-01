import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

import '../../../../environment.dart';
import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_constants.dart';
import '../widget/button/list_button.dart';
import '../widget/loading_indicator.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_app_bar.dart';

const double _kContextLogoSpacing = 40;

/// A reusable widget for displaying a loading state with optional cancellation.
///
/// This page includes a customizable title, description, app bar, and contextual image.
/// A central loading indicator is shown by default, and a cancel button is displayed
/// at the bottom if [onCancel] is provided. The widget manages accessibility focus
/// for the title to ensure it's announced when the widget appears on screen.
class GenericLoadingPage extends StatefulWidget {
  /// The main title shown at the top of the loading page, displayed in large text and centered.
  final String title;

  /// Supporting text displayed below the title, centered on the page.
  final String description;

  /// Optional callback triggered when the cancel button is pressed.
  ///
  /// If null, no cancel button is shown.
  final VoidCallback? onCancel;

  /// Text displayed in the cancel button.
  ///
  /// Defaults to [l10n.generalCancelCta] if not provided.
  final String? cancelCta;

  /// App bar (e.g. [WalletAppBar]) shown at the top of the page.
  ///
  /// Used to display the progress of a multi-step flow if provided.
  final PreferredSizeWidget? appBar;

  /// Optional contextual image (e.g., a logo) displayed above the title.
  ///
  /// Typically implemented as [Image.asset(WalletAssets.logo_wallet, height: 64, width: 64)].
  final Widget? contextImage;

  /// The loading indicator widget that shows progress during the operation.
  ///
  /// Defaults to [LoadingIndicator] but can be customized if needed.
  final Widget loadingIndicator;

  /// Controls whether the title automatically requests accessibility focus.
  ///
  /// Set to false only if the page has its own focus management.
  final bool requestAccessibilityFocus;

  const GenericLoadingPage({
    required this.title,
    required this.description,
    this.onCancel,
    this.cancelCta,
    this.appBar,
    this.requestAccessibilityFocus = true,
    this.contextImage,
    this.loadingIndicator = const LoadingIndicator(),
    super.key,
  });

  @override
  State<GenericLoadingPage> createState() => _GenericLoadingPageState();
}

class _GenericLoadingPageState extends State<GenericLoadingPage> {
  final GlobalKey _titleKey = GlobalKey();

  @override
  void initState() {
    super.initState();
    if (widget.requestAccessibilityFocus) {
      // Using addPostFrameCallback because changing focus need to wait for the widget to finish rendering.
      WidgetsBinding.instance.addPostFrameCallback((_) async {
        /// Because [GenericLoadingPage] often lives within a [FakePagingAnimatedSwitcher]. We delay moving the focus by
        /// an extra [kDefaultAnimationDuration] so that any animations can settle (this allows the focus change
        /// to behave properly on iOS as well).
        if (!Environment.isTest) await Future.delayed(kDefaultAnimationDuration);
        _titleKey.currentContext?.findRenderObject()?.sendSemanticsEvent(const FocusSemanticEvent());
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: widget.appBar,
      body: SizedBox(
        width: double.infinity,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Expanded(
              flex: context.isLandscape ? 3 : 2,
              child: SingleChildScrollView(
                reverse: true,
                /* make sure it's bottom aligned */
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.end,
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    widget.contextImage ?? const SizedBox.shrink(),
                    SizedBox(height: widget.contextImage == null ? 0 : _kContextLogoSpacing),
                    TitleText(
                      widget.title,
                      style: BaseWalletTheme.headlineExtraSmallTextStyle,
                      textAlign: TextAlign.center,
                      key: _titleKey,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      widget.description,
                      style: context.textTheme.bodyLarge,
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 24),
                  ],
                ),
              ),
            ),
            widget.loadingIndicator,
            Expanded(
              flex: 2,
              child: Column(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  _buildOptionalCancelButton(context),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildOptionalCancelButton(BuildContext context) {
    if (widget.onCancel == null) return const SizedBox.shrink();
    return SafeArea(
      left: false,
      right: false,
      child: ListButton(
        icon: const Icon(Icons.block_outlined),
        onPressed: widget.onCancel,
        dividerSide: DividerSide.top,
        mainAxisAlignment: MainAxisAlignment.center,
        text: Text(widget.cancelCta ?? context.l10n.generalCancelCta),
      ),
    );
  }
}
