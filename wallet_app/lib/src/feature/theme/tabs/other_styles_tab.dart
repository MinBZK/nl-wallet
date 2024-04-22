import 'package:flutter/material.dart';

import '../../../domain/model/app_image_data.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/missing_attribute.dart';
import '../../../domain/model/attribute/ui_attribute.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/sheet/explanation_sheet.dart';
import '../../common/sheet/help_sheet.dart';
import '../../common/widget/activity_summary.dart';
import '../../common/widget/animated_linear_progress_indicator.dart';
import '../../common/widget/attribute/attribute_row.dart';
import '../../common/widget/bullet_list.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/button/icon/back_icon_button.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/card/shared_attributes_card.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/fade_in_at_offset.dart';
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
import '../../disclosure/widget/disclosure_stop_sheet.dart';
import '../theme_screen.dart';

final _kSampleCardFront = CardFront(
  title: 'Sample Card'.untranslated,
  backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
  theme: CardFrontTheme.dark,
  info: 'Info'.untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  subtitle: 'Subtitle'.untranslated,
);

final _kSampleAttributes = [
  DataAttribute(
    key: 'key1',
    label: 'Sample #1'.untranslated,
    value: const StringValue('1'),
    sourceCardDocType: 'sourceCardDocType',
  ),
  DataAttribute(
    key: 'key2',
    label: 'Sample #2'.untranslated,
    value: const StringValue('2'),
    sourceCardDocType: 'sourceCardDocType',
  )
];

final _kSampleCard = WalletCard(
  id: 'id',
  docType: 'docType',
  front: _kSampleCardFront,
  attributes: _kSampleAttributes,
  issuer: _kSampleOrganization,
);

final _kSampleOrganization = Organization(
  id: 'id',
  legalName: 'Organization Legal Name'.untranslated,
  displayName: 'Organization Display Name'.untranslated,
  category: 'Category'.untranslated,
  description: 'Organization description'.untranslated,
  logo: const AppAssetImage(WalletAssets.logo_rijksoverheid),
);

final _kSampleOperationAttribute = OperationTimelineAttribute(
  dateTime: DateTime.now(),
  organization: _kSampleOrganization,
  dataAttributes: const [],
  status: OperationStatus.issued,
  card: _kSampleCard,
);

final _kSampleInteractionAttribute = InteractionTimelineAttribute(
  dateTime: DateTime.now(),
  organization: _kSampleOrganization,
  dataAttributes: const [],
  status: InteractionStatus.success,
  policy: const Policy(
    storageDuration: Duration(days: 90),
    dataPurpose: 'Kaart uitgifte',
    dataIsShared: false,
    deletionCanBeRequested: true,
    privacyPolicyUrl: 'https://www.example.org',
  ),
  requestPurpose: 'Kaart uitgifte'.untranslated,
);

class OtherStylesTab extends StatelessWidget {
  const OtherStylesTab({super.key});

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
        const ThemeSectionSubHeader(title: 'Wallet App Bar + FadeInAtOffset'),
        TextButton(
          onPressed: () => _showWalletAppBarPageWithFadeInTitle(context),
          child: const Text('WalletAppBar + FadeInAtOffset'),
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
        const ThemeSectionSubHeader(title: 'Disclosure Stop Sheet'),
        TextButton(
          onPressed: () {
            DisclosureStopSheet.show(
              context,
              organizationName: 'Organization name'.untranslated,
            );
          },
          child: const Text('Disclosure Stop Sheet'),
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
            _kSampleCard,
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
          _kSampleCard,
          _kSampleCard,
        ]),
        const ThemeSectionSubHeader(title: 'SharedWalletCard'),
        SharedAttributesCard(
          card: _kSampleCard,
          attributes: _kSampleCard.attributes,
        ),
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
            organization: _kSampleOrganization,
            dataAttributes: const [],
            status: InteractionStatus.success,
            policy: const Policy(
              storageDuration: Duration(days: 90),
              dataPurpose: 'Kaart uitgifte',
              dataIsShared: false,
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
            id: 'id',
            docType: 'docType',
            front: _kSampleCardFront,
            attributes: const [],
            issuer: _kSampleOrganization,
          ),
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
        const ThemeSectionSubHeader(title: 'ActivitySummary'),
        ActivitySummary(
          attributes: [
            _kSampleOperationAttribute,
            _kSampleOperationAttribute,
            _kSampleInteractionAttribute,
            _kSampleInteractionAttribute,
            _kSampleInteractionAttribute,
          ],
        ),
      ],
    );
  }

  void _showSliverWalletAppBarPage(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) {
          return const Scaffold(
            body: CustomScrollView(
              slivers: [
                SliverWalletAppBar(
                  title: 'Sliver App Bar',
                  progress: FlowProgress(currentStep: 2, totalSteps: 3),
                  leading: BackIconButton(),
                  actions: [HelpIconButton()],
                ),
                SliverFillRemaining(
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
            appBar: const WalletAppBar(
              title: Text('WalletAppBar'),
              progress: FlowProgress(currentStep: 2, totalSteps: 8),
              leading: BackIconButton(),
              actions: [HelpIconButton()],
            ),
            body: ListView.builder(
              itemBuilder: (context, index) {
                if (index == 2) {
                  return Container(
                    padding: const EdgeInsets.all(12),
                    alignment: Alignment.center,
                    child: const Text(
                      'This is a more static variant of the custom AppBar without collapse effect',
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

  void _showWalletAppBarPageWithFadeInTitle(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) {
          return Scaffold(
            appBar: const WalletAppBar(
              title: FadeInAtOffset(
                appearOffset: 50,
                visibleOffset: 150,
                child: Text('FadeInAtOffset'),
              ),
              progress: FlowProgress(currentStep: 2, totalSteps: 8),
              leading: BackIconButton(),
              actions: [HelpIconButton()],
            ),
            body: ListView.builder(
              itemBuilder: (context, index) {
                if (index == 2) {
                  return Container(
                    height: 100,
                    padding: const EdgeInsets.all(12),
                    alignment: Alignment.center,
                    child: const Text(
                      'This is the Static WalletAppBar combined with a '
                      'FadeInAtOffset to recreate the collapse and show title effect.',
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
