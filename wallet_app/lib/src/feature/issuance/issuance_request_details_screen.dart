import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/disclosure/disclose_card_request.dart';
import '../../domain/model/organization.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/object_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_constants.dart';
import '../check_attributes/check_attributes_screen.dart';
import '../common/builder/request_detail_common_builders.dart';
import '../common/sheet/select_card_sheet.dart';
import '../common/widget/animation/animated_card_switcher.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/list_button.dart';
import '../common/widget/card/shared_attributes_card.dart';
import '../common/widget/list/list_item.dart';
import '../common/widget/menu_item.dart';
import '../common/widget/organization/organization_logo.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import '../error/error_page.dart';
import '../info/info_screen.dart';
import '../organization/detail/organization_detail_screen.dart';
import 'bloc/issuance_bloc.dart';

class IssuanceRequestDetailsScreen extends StatelessWidget {
  const IssuanceRequestDetailsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(
          context.bloc.relyingParty == null
              ? ''
              : context.l10n.requestDetailScreenAltTitle(context.bloc.relyingParty!.displayName.l10nValue(context)),
        ),
      ),
      body: SafeArea(
        child: BlocBuilder<IssuanceBloc, IssuanceState>(
          builder: (context, state) {
            switch (state) {
              case IssuanceCheckOrganization():
                return _buildContent(context, state);
              default:
                return _buildError(context);
            }
          },
        ),
      ),
    );
  }

  Widget _buildError(BuildContext context) {
    return ErrorPage.generic(
      context,
      onPrimaryActionPressed: () => Navigator.pop(context),
      style: ErrorCtaStyle.close,
    );
  }

  Widget _buildContent(BuildContext context, IssuanceCheckOrganization state) {
    return Column(
      children: [
        Expanded(
          child: WalletScrollbar(
            child: ListView(
              children: [
                Padding(
                  padding: kDefaultTitlePadding,
                  child: TitleText(
                    context.l10n.requestDetailScreenAltTitle(
                      state.organization.displayName.l10nValue(context),
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                _buildOrganization(
                  context,
                  state.organization,
                  DividerSide.top,
                ),
                RequestDetailCommonBuilders.buildPurpose(
                  context,
                  purpose: state.purpose,
                  side: DividerSide.top,
                ),
                _buildCardRequests(context, state.cardRequests),
                RequestDetailCommonBuilders.buildPolicy(
                  context,
                  organization: state.organization,
                  policy: state.policy,
                  side: DividerSide.top,
                ),
                const Divider(),
                const SizedBox(height: 24),
              ],
            ),
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  Widget _buildOrganization(BuildContext context, Organization organization, DividerSide side) {
    return MenuItem(
      leftIcon: OrganizationLogo(image: organization.logo, size: kMenuItemNormalIconSize),
      dividerSide: side,
      label: Text(
        context.l10n.requestDetailScreenAboutOrganizationCta(
          organization.displayName.l10nValue(context),
        ),
      ),
      subtitle: Text(organization.category?.l10nValue(context) ?? '').takeIf((_) => organization.category != null),
      onPressed: () => OrganizationDetailScreen.showPreloaded(
        context,
        organization,
        sharedDataWithOrganizationBefore: false,
      ),
    );
  }

  Widget _buildCardRequests(BuildContext context, List<DiscloseCardRequest> cardRequests) {
    final totalNrOfAttributes = cardRequests.map((it) => it.selection.attributes).flattened.length;
    final String title = context.l10n.requestDetailsScreenAttributesTitle;
    final subtitle = context.l10n.requestDetailsScreenAttributesSubtitle(totalNrOfAttributes);

    final header = ListItem(
      label: Text.rich(title.toTextSpan(context)),
      subtitle: Text.rich(subtitle.toTextSpan(context)),
      icon: const Icon(Icons.credit_card_outlined),
      style: ListItemStyle.vertical,
      dividerSide: DividerSide.top,
    );

    final attributeWidgets = cardRequests.map((request) {
      return AnimatedCardSwitcher(
        enableAnimation: !context.isScreenReaderEnabled,
        child: SharedAttributesCard(
          key: ValueKey(request.selection.attestationId),
          card: request.selection,
          onPressed: () => _onShowCardDetailsPressed(context, request),
          onChangeCardPressed: request.hasAlternatives ? () => _showSelectAlternativeCardSheet(context, request) : null,
        ),
      );
    }).toList();

    return Column(
      children: [
        header,
        ListView.separated(
          physics: const NeverScrollableScrollPhysics(),
          shrinkWrap: true,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          itemBuilder: (c, i) => attributeWidgets[i],
          separatorBuilder: (c, i) => const SizedBox(height: 16),
          itemCount: attributeWidgets.length,
        ),
        const SizedBox(height: 24),
      ],
    );
  }

  void _onShowCardDetailsPressed(BuildContext context, DiscloseCardRequest request) {
    CheckAttributesScreen.showWithAlternatives(
      context,
      selection: request.selection,
      cards: request.candidates,
      onDataIncorrectPressed: () => InfoScreen.showDetailsIncorrect(context),
      onAlternativeCardSelected: (card) {
        final updatedRequest = request.select(card);
        context.bloc.add(IssuanceAlternativeCardSelected(updatedRequest));
      },
    );
  }

  Future<void> _showSelectAlternativeCardSheet(BuildContext context, DiscloseCardRequest request) async {
    final selection = await SelectCardSheet.show(context, candidates: request.alternatives);
    if (selection != null && context.mounted) {
      final updatedRequest = request.select(selection);
      context.bloc.add(IssuanceAlternativeCardSelected(updatedRequest));
    }
  }

  static Future<void> show(
    BuildContext context, {
    required IssuanceBloc bloc,
  }) {
    assert(
      bloc.state is IssuanceCheckOrganization,
      'IssuanceRequestDetailsScreen should be shown when bloc is in the expected state',
    );
    return Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) => BlocProvider.value(
          value: bloc,
          child: const IssuanceRequestDetailsScreen(),
        ),
      ),
    );
  }
}

extension _IssuanceRequestDetailsScreenExtensions on BuildContext {
  IssuanceBloc get bloc => read<IssuanceBloc>();
}
