import 'package:flutter/material.dart';

import '../../../domain/model/app_image_data.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/missing_attribute.dart';
import '../../../domain/model/attribute/ui_attribute.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/sheet/explanation_sheet.dart';
import '../../common/sheet/help_sheet.dart';
import '../../common/widget/animated_linear_progress_indicator.dart';
import '../../common/widget/attribute/attribute_row.dart';
import '../../common/widget/bullet_list.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/button/wallet_back_button.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/history/timeline_attribute_row.dart';
import '../../common/widget/history/timeline_section_header.dart';
import '../../common/widget/icon_row.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/loading_indicator.dart';
import '../../common/widget/numbered_list.dart';
import '../../common/widget/pin_field_demo.dart';
import '../../common/widget/pin_header.dart';
import '../../common/widget/policy/policy_row.dart';
import '../../common/widget/policy/policy_section.dart';
import '../../common/widget/select_card_row.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/stacked_wallet_cards.dart';
import '../../common/widget/status_icon.dart';
import '../../common/widget/version_text.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../common/widget/wallet_logo.dart';
import '../../disclosure/widget/card_attribute_row.dart';
import '../theme_screen.dart';

final _kSampleCardFront = CardFront(
  title: 'Sample Card'.untranslated,
  backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
  theme: CardFrontTheme.dark,
  info: 'Info'.untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  subtitle: 'Subtitle'.untranslated,
);

class OtherStylesTab extends StatelessWidget {
  const OtherStylesTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 32),
      children: [
        _buildAppBarSection(context),
        _buildSheetSection(context),
        _buildAttributeSection(context),
        _buildCardSection(context),
        _buildHistorySection(context),
        _buildPolicySection(context),
        _buildMiscellaneousSection(context),
      ],
    );
  }

  Widget _buildAppBarSection(BuildContext context) {
    return Column(
      children: [
        const ThemeSectionHeader(title: 'App Bars'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'Sliver Wallet App Bar'),
        TextButton(
          onPressed: () => _showSliverWalletAppBarPage(context),
          child: const Text('SliverWalletAppBar'),
        ),
        const ThemeSectionSubHeader(title: 'Wallet App Bar'),
        TextButton(
          onPressed: () => _showWalletAppBarPage(context),
          child: const Text('WalletAppBar'),
        ),
      ],
    );
  }

  Widget _buildSheetSection(BuildContext context) {
    return Column(
      children: [
        const ThemeSectionHeader(title: 'Sheets'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'Explanation Sheet'),
        TextButton(
          onPressed: () {
            ExplanationSheet.show(
              context,
              title: 'Title goes here',
              description: 'Description goes here. This is a demo of the ExplanationSheet!',
              closeButtonText: 'close',
            );
          },
          child: const Text('Explanation Sheet'),
        ),
        const ThemeSectionSubHeader(title: 'Confirm Action Sheet'),
        TextButton(
          onPressed: () {
            ConfirmActionSheet.show(
              context,
              title: 'Title goes here',
              description: 'Description goes here. This is a demo of the ConfirmActionSheet!',
              cancelButtonText: 'cancel',
              confirmButtonText: 'confirm',
            );
          },
          child: const Text('Confirm Action Sheet'),
        ),
        const ThemeSectionSubHeader(title: '(Error) Help Sheet'),
        TextButton(
          onPressed: () {
            HelpSheet.show(
              context,
              errorCode: 'xxyyzz',
              supportCode: '1337',
            );
          },
          child: const Text('(Error) Help Sheet'),
        ),
      ],
    );
  }

  Widget _buildAttributeSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'Attributes'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'DataAttributeRow - Type Text'),
        AttributeRow(
          attribute: DataAttribute.untranslated(
            value: const StringValue('This is a DataAttributeRow with type text'),
            label: 'Label',
            sourceCardDocType: 'id',
            key: 'mock.other',
          ),
        ),
        const ThemeSectionSubHeader(title: 'RequestedAttributeRow'),
        AttributeRow(
          attribute: MissingAttribute.untranslated(
            label: 'This is a RequestedAttributeRow',
            key: 'mock.other',
          ),
        ),
        const ThemeSectionSubHeader(title: 'UiAttributeRow'),
        AttributeRow(
          attribute: UiAttribute.untranslated(
            value: const StringValue('This is a UiAttributeRow'),
            key: 'mock.other',
            label: 'Label',
            icon: Icons.remove_red_eye,
          ),
        ),
        const ThemeSectionSubHeader(title: 'CardAttributeRow'),
        CardAttributeRow(
          entry: MapEntry(
            WalletCard(id: 'id', docType: 'docType', front: _kSampleCardFront, attributes: const [], issuerId: ''),
            [
              DataAttribute.untranslated(
                label: 'Voornaam',
                value: const StringValue(''),
                sourceCardDocType: '',
                key: '',
              ),
              DataAttribute.untranslated(
                label: 'Achternaam',
                value: const StringValue(''),
                sourceCardDocType: '',
                key: '',
              ),
              DataAttribute.untranslated(
                label: 'Postcode',
                value: const StringValue(''),
                sourceCardDocType: '',
                key: '',
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildCardSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'Cards'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'WalletCardItem'),
        const WalletCardItem(
          title: 'Card Title',
          background: WalletAssets.svg_rijks_card_bg_dark,
          brightness: Brightness.dark,
          subtitle1: 'Card subtitle1',
          subtitle2: 'Card subtitle2',
          logo: WalletAssets.logo_card_rijksoverheid,
          holograph: WalletAssets.svg_rijks_card_holo,
        ),
        const ThemeSectionSubHeader(title: 'StackedWalletCards'),
        StackedWalletCards(cards: [
          WalletCard(
              id: 'id', docType: 'docType', issuerId: 'issuerId', front: _kSampleCardFront, attributes: const []),
          WalletCard(
              id: 'id', docType: 'docType', issuerId: 'issuerId', front: _kSampleCardFront, attributes: const []),
        ]),
      ],
    );
  }

  Widget _buildHistorySection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'History'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'TimelineAttributeRow'),
        TimelineAttributeRow(
          attribute: InteractionTimelineAttribute(
            dateTime: DateTime.now(),
            organization: Organization(
              id: 'id',
              legalName: 'Organization Name'.untranslated,
              category: 'Category'.untranslated,
              displayName: 'This is a TimelineAttributeRow'.untranslated,
              description: 'Organization description'.untranslated,
              logo: const AppAssetImage(WalletAssets.logo_rijksoverheid),
            ),
            dataAttributes: const [],
            status: InteractionStatus.success,
            policy: const Policy(
              storageDuration: Duration(days: 90),
              dataPurpose: 'Kaart uitgifte',
              dataIsShared: false,
              dataIsSignature: false,
              dataContainsSingleViewProfilePhoto: false,
              deletionCanBeRequested: true,
              privacyPolicyUrl: 'https://www.example.org',
            ),
            requestPurpose: 'Kaart uitgifte'.untranslated,
          ),
          onPressed: () {},
        ),
        const ThemeSectionSubHeader(title: 'TimelineSectionHeader'),
        TimelineSectionHeader(dateTime: DateTime.now()),
      ],
    );
  }

  Widget _buildPolicySection(BuildContext context) {
    return const Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ThemeSectionHeader(title: 'Policy'),
        SizedBox(height: 12),
        ThemeSectionSubHeader(title: 'PolicyRow'),
        PolicyRow(icon: Icons.alarm, title: 'This is a Policy Row'),
        ThemeSectionSubHeader(title: 'PolicySection'),
        PolicySection(
          Policy(
            storageDuration: Duration(days: 90),
            dataPurpose: 'Kaart uitgifte',
            dataIsShared: false,
            dataIsSignature: false,
            dataContainsSingleViewProfilePhoto: false,
            deletionCanBeRequested: true,
            privacyPolicyUrl: 'https://www.example.org',
          ),
        ),
      ],
    );
  }

  Widget _buildMiscellaneousSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'Miscellaneous'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'AnimatedLinearProgressIndicator'),
        const AnimatedLinearProgressIndicator(progress: 0.3),
        const ThemeSectionSubHeader(title: 'AnimatedVisibilityBackButton'),
        const AnimatedVisibilityBackButton(visible: true),
        const ThemeSectionSubHeader(title: 'CenteredLoadingIndicator'),
        const CenteredLoadingIndicator(),
        const ThemeSectionSubHeader(title: 'LoadingIndicator'),
        const LoadingIndicator(),
        const ThemeSectionSubHeader(title: 'PinHeader'),
        const PinHeader(title: 'Title', description: 'Description', hasError: false),
        const ThemeSectionSubHeader(title: 'SelectCardRow'),
        SelectCardRow(
          onCardSelectionToggled: (_) {},
          card: WalletCard(
              id: 'id', docType: 'docType', issuerId: 'issuerId', front: _kSampleCardFront, attributes: const []),
          isSelected: true,
        ),
        const ThemeSectionSubHeader(title: 'StatusIcon'),
        const StatusIcon(icon: Icons.ac_unit),
        const ThemeSectionSubHeader(title: 'VersionText'),
        const VersionText(),
        const ThemeSectionSubHeader(title: 'WalletLogo'),
        const WalletLogo(size: 64),
        const ThemeSectionSubHeader(title: 'IconRow'),
        const IconRow(
          icon: Icon(Icons.ac_unit),
          text: Text('IconRow'),
        ),
        const ThemeSectionSubHeader(title: 'InfoRow'),
        const InfoRow(
          icon: Icons.ac_unit,
          title: Text('Title'),
          subtitle: Text('Subtitle'),
        ),
        const ThemeSectionSubHeader(title: 'PinField'),
        const PinFieldDemo(),
        const ThemeSectionSubHeader(title: 'BulletList'),
        const BulletList(
          items: ['Item 1', 'Item 2', 'Item 3'],
          icon: Icons.ac_unit_outlined,
        ),
        const ThemeSectionSubHeader(title: 'NumberedList'),
        const NumberedList(
          items: ['Item', 'Item', 'Item'],
        ),
      ],
    );
  }

  void _showSliverWalletAppBarPage(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) {
          return Scaffold(
            body: CustomScrollView(
              slivers: [
                SliverWalletAppBar(
                  title: 'Sliver App Bar',
                  progress: 0.65,
                  leading: const WalletBackButton(),
                  actions: [IconButton(onPressed: () {}, icon: const Icon(Icons.help_outline_rounded))],
                ),
                const SliverFillRemaining(
                  child: Padding(
                    padding: EdgeInsets.symmetric(horizontal: 16),
                    child: Text('Scroll this page to see the collapsing effect'),
                  ),
                )
              ],
            ),
          );
        },
      ),
    );
  }

  void _showWalletAppBarPage(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) {
          return Scaffold(
            appBar: WalletAppBar(
              title: const Text('WalletAppBar'),
              progress: 0.55,
              leading: const WalletBackButton(),
              actions: [IconButton(onPressed: () {}, icon: const Icon(Icons.help_outline_rounded))],
            ),
            body: ListView.builder(
              itemBuilder: (context, index) {
                if (index == 2) {
                  return Container(
                    padding: const EdgeInsets.all(12),
                    alignment: Alignment.center,
                    child: const Text(
                      'This is a more static variant of the custom AppBar and does not support the collapse effect',
                      textAlign: TextAlign.center,
                    ),
                  );
                }
                return Container(
                  height: 100,
                  color: index.isOdd ? Colors.greenAccent : Colors.transparent,
                );
              },
              itemCount: 50,
            ),
          );
        },
      ),
    );
  }
}
