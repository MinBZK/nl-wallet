import 'dart:io';

import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:intl/locale.dart' as intl;

import '../../../environment.dart';
import '../../../l10n/generated/app_localizations.dart';
import '../../theme/base_wallet_theme.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/object_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/spacer/sliver_divider.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'bloc/change_language_bloc.dart';

class ChangeLanguageScreen extends StatelessWidget {
  const ChangeLanguageScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.changeLanguageScreenTitle),
        leading: const BackIconButton(
          key: Key('changeLanguageScreenBackCta'),
        ),
      ),
      key: const Key('changeLanguageScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: kDefaultTitlePadding,
                        child: TitleText(context.l10n.changeLanguageScreenTitle),
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
          separatorBuilder: (c, i) => const Divider(),
          itemBuilder: (c, i) {
            final language = state.availableLanguages[i];
            final isSelectedLanguage = state.availableLanguages[i].locale == state.selectedLocale;
            return Semantics(
              inMutuallyExclusiveGroup: true,
              selected: isSelectedLanguage,
              button: !isSelectedLanguage,
              onTap: isSelectedLanguage
                  ? null
                  : () {
                      final changeLocaleEvent = ChangeLanguageLocaleSelected(language.locale);
                      context.read<ChangeLanguageBloc>().add(changeLocaleEvent);
                    },
              excludeSemantics: true,
              attributedLabel: AttributedString(
                language.name,
                attributes: [
                  LocaleStringAttribute(
                    range: language.name.fullRange,
                    locale: language.locale,
                  ),
                ],
              ),
              onTapHint: _lookupSystemLocalizations(context).generalWCAGChangeLanguage,
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

  /// Looks up the system Locale (so NOT the locale that is currently selected in the app) and
  /// uses that to provide the corresponding [AppLocalizations]. Falls back to the default
  /// [AppLocalizations] when the system's Locale does not match any of the supportedLocales.
  AppLocalizations _lookupSystemLocalizations(BuildContext context) {
    final locale = intl.Locale.tryParse(Platform.localeName);
    final supportedLocale = AppLocalizations.supportedLocales.firstWhereOrNull(
      (supported) => supported.languageCode == locale?.languageCode,
    );
    return supportedLocale == null ? context.l10n : lookupAppLocalizations(supportedLocale);
  }

  TextStyle _getRowTextStyle(BuildContext context, bool isSelected) {
    final baseStyle = context.textTheme.bodyLarge ?? const TextStyle();
    if (!isSelected) return baseStyle;
    return baseStyle.copyWith(
      color: context.colorScheme.primary,
      fontVariations: [BaseWalletTheme.fontVariationBold],
    );
  }

  Widget _buildContentSliver(BuildContext context) {
    return BlocConsumer<ChangeLanguageBloc, ChangeLanguageState>(
      listenWhen: (previous, current) {
        // This indicates the language was updated
        return previous is ChangeLanguageSuccess && current is ChangeLanguageSuccess;
      },
      listener: (context, state) async {
        if (state is ChangeLanguageSuccess) {
          final language = state.availableLanguages.firstWhereOrNull(
            (language) => language.locale.languageCode == state.selectedLocale.languageCode,
          );
          // Avoid conflicting with the announcement of the (now) selected language
          await Future.delayed(Environment.isTest ? Duration.zero : const Duration(milliseconds: 1500));
          await language?.let((it) => _announceNewLanguage(context, it));
        }
      },
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

  Future<void> _announceNewLanguage(BuildContext context, Language language) async {
    final systemL10n = _lookupSystemLocalizations(context);
    final announcement = systemL10n.generalWCAGLanguageUpdatedAnnouncement(language.name);
    await SemanticsService.announce(announcement, TextDirection.ltr);
  }
}
