import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/sliver_divider.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'bloc/change_language_bloc.dart';

class ChangeLanguageScreen extends StatelessWidget {
  const ChangeLanguageScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('changeLanguageScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.changeLanguageScreenTitle,
                      scrollController: PrimaryScrollController.maybeOf(context),
                      leading: const BackIconButton(
                        key: Key('changeLanguageScreenBackCta'),
                      ),
                    ),
                    const SliverSizedBox(height: 12),
                    _buildContentSliver(context),
                  ],
                ),
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildSuccessSliver(BuildContext context, ChangeLanguageSuccess state) {
    return SliverMainAxisGroup(
      slivers: [
        const SliverDivider(),
        SliverList.separated(
          separatorBuilder: (c, i) => const Divider(height: 1),
          itemBuilder: (c, i) {
            final language = state.availableLanguages[i];
            final isSelectedLanguage = state.availableLanguages[i].locale == state.selectedLocale;
            return Semantics(
              selected: isSelectedLanguage,
              onTap: isSelectedLanguage
                  ? null
                  : () {
                      final changeLocaleEvent = ChangeLanguageLocaleSelected(language.locale);
                      context.read<ChangeLanguageBloc>().add(changeLocaleEvent);
                    },
              excludeSemantics: true,
              label: language.name,
              onTapHint: context.l10n.generalWCAGLogoutAnnouncement,
              child: InkWell(
                onTap: isSelectedLanguage
                    ? null
                    : () {
                        final changeLocaleEvent = ChangeLanguageLocaleSelected(language.locale);
                        context.read<ChangeLanguageBloc>().add(changeLocaleEvent);
                      },
                child: Container(
                  key: ValueKey(language),
                  constraints: const BoxConstraints(minHeight: 72),
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                  alignment: Alignment.centerLeft,
                  child: Row(
                    children: [
                      Expanded(
                        child: AnimatedDefaultTextStyle(
                          duration: kDefaultAnimationDuration,
                          style: _getRowTextStyle(context, isSelectedLanguage),
                          child: Text(language.name),
                        ),
                      ),
                      AnimatedOpacity(
                        opacity: isSelectedLanguage ? 1 : 0,
                        duration: kDefaultAnimationDuration,
                        child: Icon(
                          Icons.check,
                          color: context.colorScheme.primary,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            );
          },
          itemCount: state.availableLanguages.length,
        ),
        const SliverDivider(),
      ],
    );
  }

  TextStyle _getRowTextStyle(BuildContext context, bool isSelected) {
    final baseStyle = context.textTheme.bodyLarge ?? const TextStyle();
    if (!isSelected) return baseStyle;
    return baseStyle.copyWith(color: context.colorScheme.primary, fontWeight: FontWeight.bold);
  }

  Widget _buildContentSliver(BuildContext context) {
    return BlocBuilder<ChangeLanguageBloc, ChangeLanguageState>(
      builder: (context, state) {
        return switch (state) {
          ChangeLanguageInitial() => _buildLoadingSliver(),
          ChangeLanguageSuccess() => _buildSuccessSliver(context, state),
        };
      },
    );
  }

  Widget _buildLoadingSliver() {
    return const SliverFillRemaining(
      hasScrollBody: false,
      child: CenteredLoadingIndicator(),
    );
  }
}
