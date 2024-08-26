import 'package:fimber/fimber.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/formatter/country_code_formatter.dart';
import '../../../util/launch_util.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/focus_builder.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/wallet_scrollbar.dart';
import 'argument/organization_detail_screen_argument.dart';
import 'bloc/organization_detail_bloc.dart';

class OrganizationDetailScreen extends StatelessWidget {
  static OrganizationDetailScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return OrganizationDetailScreenArgument.fromMap(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError(
        'Make sure to pass in [organizationDetailScreenArgument] when opening the organizationDetailScreen',
      );
    }
  }

  final VoidCallback? onReportIssuePressed;

  const OrganizationDetailScreen({
    this.onReportIssuePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildBody(),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildBody() {
    return BlocBuilder<OrganizationDetailBloc, OrganizationDetailState>(
      builder: (context, state) {
        final content = switch (state) {
          OrganizationDetailInitial() => _buildLoadingSliver(),
          OrganizationDetailFailure() => _buildOrganizationDetailFailure(context, state),
          OrganizationDetailSuccess() => _buildOrganizationDetailLoaded(context, state),
        };
        return WalletScrollbar(
          child: CustomScrollView(
            slivers: [
              SliverWalletAppBar(
                title: _resolveTitle(context),
                scrollController: PrimaryScrollController.maybeOf(context),
                actions: const [HelpIconButton()],
              ),
              content,
            ],
          ),
        );
      },
    );
  }

  String _resolveTitle(BuildContext context) {
    final state = context.read<OrganizationDetailBloc>().state;
    if (state is! OrganizationDetailSuccess) return '';
    return context.l10n.organizationDetailScreenTitle(state.organization.displayName.l10nValue(context));
  }

  Widget _buildLoadingSliver() {
    return const SliverFillRemaining(
      hasScrollBody: false,
      child: CenteredLoadingIndicator(),
    );
  }

  Widget _buildOrganizationDetailFailure(BuildContext context, OrganizationDetailFailure state) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Center(
        child: TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text.rich(context.l10n.generalBottomBackCta.toTextSpan(context)),
        ),
      ),
    );
  }

  Widget _buildOrganizationDetailLoaded(BuildContext context, OrganizationDetailSuccess state) {
    final items = _buildInfoSectionItems(context, state.organization);
    return SliverList.list(
      children: [
        const SizedBox(height: 24),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: _buildHeaderSection(context, state.organization),
        ),
        const SizedBox(height: 24),
        if (state.sharedDataWithOrganizationBefore) ...[
          const Divider(height: 1),
          _buildInteractionRow(context, state),
        ],
        const Divider(height: 1),
        ListView.separated(
          physics: const NeverScrollableScrollPhysics(),
          padding: const EdgeInsets.symmetric(vertical: 16),
          shrinkWrap: true,
          itemBuilder: (c, i) => items[i],
          separatorBuilder: (c, i) => const SizedBox(height: 24),
          itemCount: items.length,
        ),
        onReportIssuePressed == null
            ? const SizedBox()
            : ListButton(
                text: Text.rich(context.l10n.organizationDetailScreenReportIssueCta.toTextSpan(context)),
                onPressed: () {
                  Navigator.pop(context);
                  onReportIssuePressed?.call();
                },
              ),
        const SizedBox(height: 24),
      ],
    );
  }

  Widget _buildHeaderSection(BuildContext context, Organization organization) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ExcludeSemantics(
          child: OrganizationLogo(image: organization.logo, size: 64, fixedRadius: 12),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Text.rich(
            organization.description?.l10nSpan(context) ?? ''.toTextSpan(context),
            textAlign: TextAlign.start,
            style: context.textTheme.bodyLarge,
          ),
        ),
      ],
    );
  }

  List<Widget> _buildInfoSectionItems(BuildContext context, Organization organization) {
    final country = CountryCodeFormatter.format(organization.countryCode);
    return [
      _buildLegalNameRow(context, organization),
      _buildCategoryRow(context, organization),
      if (organization.department != null) _buildDepartmentRow(context, organization),
      if (country != null || organization.city != null) _buildLocationRow(context, country, organization),
      if (organization.webUrl != null) _buildWebUrlRow(context, organization.webUrl!),
      if (organization.privacyPolicyUrl != null) _buildPrivacyRow(context, organization.privacyPolicyUrl!),
      if (organization.kvk != null) _buildKvkRow(context, organization),
    ];
  }

  Widget _buildLegalNameRow(BuildContext context, Organization organization) {
    return _buildInfoRow(
      context,
      icon: Icons.balance_outlined,
      title: Text.rich(context.l10n.organizationDetailScreenLegalNameInfo.toTextSpan(context)),
      subtitle: Text(organization.legalName.l10nValue(context)),
    );
  }

  Widget _buildCategoryRow(BuildContext context, Organization organization) {
    return _buildInfoRow(
      context,
      icon: Icons.apartment_outlined,
      title: Text.rich(context.l10n.organizationDetailScreenCategoryInfo.toTextSpan(context)),
      subtitle: Text.rich(organization.category?.l10nSpan(context) ?? ''.toTextSpan(context)),
    );
  }

  Widget _buildDepartmentRow(BuildContext context, Organization organization) {
    return _buildInfoRow(
      context,
      icon: Icons.meeting_room_outlined,
      title: Text.rich(context.l10n.organizationDetailScreenDepartmentInfo.toTextSpan(context)),
      subtitle: Text.rich(organization.department!.l10nSpan(context)),
    );
  }

  Widget _buildLocationRow(BuildContext context, String? country, Organization organization) {
    return _buildInfoRow(
      context,
      icon: Icons.location_on_outlined,
      title: Text.rich(context.l10n.organizationDetailScreenLocationInfo.toTextSpan(context)),
      subtitle: Text.rich(_generateLocationLabel(context, country, organization.city).toTextSpan(context)),
    );
  }

  Widget _buildWebUrlRow(BuildContext context, String webUrl) {
    return _buildInfoRowWithUrl(
      context,
      icon: Icons.language_outlined,
      title: context.l10n.organizationDetailScreenWebsiteInfo,
      url: webUrl,
      semanticsLabel: '${context.l10n.organizationDetailScreenWebsiteInfo}\n$webUrl',
      onTap: () => launchUrlStringCatching(webUrl),
    );
  }

  Widget _buildPrivacyRow(BuildContext context, String privacyPolicyUrl) {
    return _buildInfoRowWithUrl(
      context,
      icon: Icons.policy_outlined,
      title: context.l10n.organizationDetailScreenPrivacyInfo,
      url: privacyPolicyUrl,
      semanticsLabel: '${context.l10n.organizationDetailScreenPrivacyInfo}\n$privacyPolicyUrl',
      onTap: () => launchUrlStringCatching(privacyPolicyUrl),
    );
  }

  Widget _buildKvkRow(BuildContext context, Organization organization) {
    final kvkRange = TextRange(start: 0, end: organization.kvk?.length ?? 0);
    final label = AttributedString(
      organization.kvk ?? '',
      attributes: [
        LocaleStringAttribute(
          range: kvkRange,
          locale: context.activeLocale,
        ),
        SpellOutStringAttribute(range: kvkRange),
      ],
    );
    return _buildInfoRow(
      context,
      icon: Icons.storefront_outlined,
      title: Text.rich(context.l10n.organizationDetailScreenKvkInfo.toTextSpan(context)),
      subtitle: Semantics(
        attributedLabel: label,
        excludeSemantics: true,
        child: Text(organization.kvk ?? ''),
      ),
    );
  }

  Widget _buildInfoRow(
    BuildContext context, {
    required IconData icon,
    required Widget title,
    required Widget subtitle,
  }) {
    /// Note: not relying on [InfoRow] widget because the styling here is a bit too custom.
    return ConstrainedBox(
      constraints: const BoxConstraints(minHeight: 44),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        mainAxisAlignment: MainAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Icon(icon, size: 24),
          ),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                DefaultTextStyle(
                  style: context.textTheme.bodySmall!,
                  child: title,
                ),
                DefaultTextStyle(
                  style: context.textTheme.bodyLarge!.copyWith(fontWeight: FontWeight.w400),
                  child: subtitle,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInfoRowWithUrl(
    BuildContext context, {
    required IconData icon,
    required String title,
    required String url,
    required String semanticsLabel,
    required VoidCallback onTap,
  }) {
    return FocusBuilder(
      onEnterPressed: onTap,
      builder: (context, hasFocus) {
        return Semantics(
          onTapHint: context.l10n.generalWCAGOpenLink,
          excludeSemantics: true,
          attributedLabel: semanticsLabel.toAttributedString(context),
          button: false,
          onTap: onTap,
          child: _buildInfoRow(
            context,
            icon: icon,
            title: Text(title),
            subtitle: Text.rich(
              TextSpan(
                text: url,
                style: context.textTheme.bodyLarge!.copyWith(
                  fontWeight: FontWeight.w400,
                  decoration: TextDecoration.underline,
                  backgroundColor: hasFocus ? context.theme.focusColor : null,
                  color: context.colorScheme.primary,
                ),
                recognizer: TapGestureRecognizer()..onTap = onTap,
              ),
            ),
          ),
        );
      },
    );
  }

  static Future<void> showPreloaded(
    BuildContext context,
    Organization organization, {
    required bool sharedDataWithOrganizationBefore,
    VoidCallback? onReportIssuePressed,
  }) {
    return Navigator.push(
      context,
      SecuredPageRoute(
        builder: (context) {
          return BlocProvider<OrganizationDetailBloc>(
            create: (BuildContext context) => OrganizationDetailBloc()
              ..add(
                OrganizationProvided(
                  organization: organization,
                  sharedDataWithOrganizationBefore: sharedDataWithOrganizationBefore,
                ),
              ),
            child: OrganizationDetailScreen(onReportIssuePressed: onReportIssuePressed),
          );
        },
      ),
    );
  }

  String _generateLocationLabel(BuildContext context, String? country, LocalizedText? city) {
    assert(country != null || city != null, 'At least one of [country, city] needs to be provided');
    final cityLabel = city?.l10nValue(context);
    if (cityLabel == null) return country!;
    if (country == null) return cityLabel;
    return '$cityLabel, $country';
  }

  Widget _buildInteractionRow(BuildContext context, OrganizationDetailSuccess state) {
    final String interaction =
        context.l10n.organizationDetailScreenSomeInteractions(state.organization.displayName.l10nValue(context));
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const Padding(
            padding: EdgeInsets.all(16),
            child: Icon(Icons.history_outlined, size: 24),
          ),
          Expanded(
            child: Text(interaction),
          ),
        ],
      ),
    );
  }
}
