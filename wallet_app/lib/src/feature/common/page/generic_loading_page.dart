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

class GenericLoadingPage extends StatefulWidget {
  /// The title shown above the loading indicator
  final String title;

  /// The description shown above the loading indicator
  final String description;

  /// The action to perform when the cancel button is pressed, button is hidden when null
  final VoidCallback? onCancel;

  /// The text shown inside the cancel button, defaults to l10n.generalCancelCta
  final String? cancelCta;

  /// Appbar (e.g. a [WalletAppBar]) to be shown at the top of the top of the loading page,
  /// useful when the loading page should also render the stepperProgress.
  final PreferredSizeWidget? appBar;

  final Widget loadingIndicator;

  final bool requestAccessibilityFocus;

  const GenericLoadingPage({
    required this.title,
    required this.description,
    this.onCancel,
    this.cancelCta,
    this.appBar,
    this.requestAccessibilityFocus = true,
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
              child: SingleChildScrollView(
                reverse: true,
                /* make sure it's bottom aligned */
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.end,
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
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
