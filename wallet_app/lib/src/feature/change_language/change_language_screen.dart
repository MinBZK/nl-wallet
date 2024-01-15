import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/wallet_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import 'bloc/change_language_bloc.dart';

class ChangeLanguageScreen extends StatelessWidget {
  const ChangeLanguageScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('changeLanguageScreen'),
      body: CustomScrollView(
        slivers: [
          SliverWalletAppBar(
            title: context.l10n.changeLanguageScreenTitle,
            leading: const WalletBackButton(
              key: Key('changeLanguageScreenBackCta'),
            ),
          ),
          const SliverSizedBox(height: 12),
          _buildContentSliver(context),
        ],
      ),
    );
  }

  Widget _buildSuccessSliver(BuildContext context, ChangeLanguageSuccess state) {
    return SliverList.builder(
      itemBuilder: (c, i) {
        if (i == state.availableLanguages.length) return const Divider(height: 1); //Draw final divider
        final language = state.availableLanguages[i];
        final isSelectedLanguage = state.availableLanguages[i].locale == state.selectedLocale;
        return InkWell(
          onTap: () {
            final changeLocaleEvent = ChangeLanguageLocaleSelected(language.locale);
            context.read<ChangeLanguageBloc>().add(changeLocaleEvent);
          },
          child: Column(
            children: [
              const Divider(height: 1),
              Container(
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
            ],
          ),
        );
      },
      itemCount: state.availableLanguages.length + 1, // +1 to Add divider
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
