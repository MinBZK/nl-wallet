import 'package:flutter/material.dart';

import '../../../theme/base_wallet_theme.dart';
import '../../../theme/wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/svg_or_image.dart';
import '../../common/widget/text/body_text.dart';

const _kTourIconSize = 32.0;

class TourBanner extends StatefulWidget {
  final EdgeInsets padding;

  const TourBanner({this.padding = EdgeInsets.zero, super.key});

  @override
  State<TourBanner> createState() => _TourBannerState();
}

class _TourBannerState extends State<TourBanner> {
  late WidgetStatesController _statesController;

  @override
  void initState() {
    super.initState();

    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(_updateState));
  }

  void _updateState() => setState(() {});

  @override
  void dispose() {
    _statesController.removeListener(_updateState);
    _statesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: widget.padding,
      child: Material(
        color: context.colorScheme.tertiaryContainer,
        shape: RoundedRectangleBorder(
          borderRadius: WalletTheme.kBorderRadius12,
          side: context.theme.elevatedButtonTheme.style?.side?.resolve(_statesController.value) ?? BorderSide.none,
        ),
        child: InkWell(
          statesController: _statesController,
          borderRadius: WalletTheme.kBorderRadius12,
          onTap: () => PlaceholderScreen.showGeneric(context),
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                _buildStoreLogo(),
                const SizedBox(width: 12),
                Expanded(
                  child: _buildTextContent(
                    context,
                    context.l10n.tourBannerTitle,
                  ),
                ),
                const SizedBox(width: 12),
                _buildForwardIcon(),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildForwardIcon() {
    return Icon(
      Icons.arrow_forward_outlined,
      size: context.theme.iconTheme.size! * (_statesController.value.isFocused ? 1.25 : 1.0),
    );
  }

  Widget _buildTextContent(BuildContext context, String title) {
    return BodyText(
      title,
      style: context.textTheme.titleMedium?.copyWith(
        decoration: _statesController.value.isFocused ? TextDecoration.underline : null,
      ),
    );
  }

  Widget _buildStoreLogo() {
    return SizedBox(
      height: _kTourIconSize,
      width: _kTourIconSize,
      child: SvgOrImage(
        asset: WalletAssets.svg_tour_icon,
      ),
    );
  }
}
