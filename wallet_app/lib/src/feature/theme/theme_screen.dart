import 'package:flutter/material.dart';
import 'tabs/button_styles_tab.dart';
import 'tabs/color_styles_tab.dart';
import 'tabs/other_styles_tab.dart';

import 'tabs/text_styles_tab.dart';

class ThemeScreen extends StatefulWidget {
  const ThemeScreen({Key? key}) : super(key: key);

  @override
  State<ThemeScreen> createState() => _ThemeScreenState();
}

class _ThemeScreenState extends State<ThemeScreen> with SingleTickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    _tabController = TabController(length: 4, vsync: this);
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Design system (draft)'),
      ),
      body: SafeArea(
        child: Column(
          children: [
            TabBar(
              onTap: (index) => setState(() => _tabController.index = index),
              controller: _tabController,
              tabs: const [
                Tab(text: 'TextStyles'),
                Tab(text: 'Buttons'),
                Tab(text: 'Colors'),
                Tab(text: 'Other'),
              ],
            ),
            Expanded(
              child: Scrollbar(
                thumbVisibility: true,
                child: _buildContent(context),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    switch (_tabController.index) {
      case 0:
        return const TextStylesTab();
      case 1:
        return const ButtonStylesTab();
      case 2:
        return const ColorStylesTab();
      case 3:
        return const OtherStylesTab();
    }
    return const Center(child: Text('Unknown tab'));
  }
}

class ThemeSectionHeader extends StatelessWidget {
  final String title;

  const ThemeSectionHeader({required this.title, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      alignment: Alignment.center,
      color: Colors.black12,
      child: Text('- $title -', style: Theme.of(context).textTheme.headlineMedium),
    );
  }
}

class ThemeSectionSubHeader extends StatelessWidget {
  final String title;

  const ThemeSectionSubHeader({required this.title, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      alignment: Alignment.centerLeft,
      color: Colors.black.withAlpha(10),
      child: Text('$title:', style: Theme.of(context).textTheme.titleSmall),
    );
  }
}
