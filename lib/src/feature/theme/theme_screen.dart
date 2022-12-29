import 'package:flutter/material.dart';

import '../common/widget/confirm_action_sheet.dart';
import '../common/widget/explanation_sheet.dart';
import '../common/widget/link_button.dart';
import '../common/widget/text_icon_button.dart';

class ThemeScreen extends StatelessWidget {
  const ThemeScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Design system (draft)'),
      ),
      body: SafeArea(
        child: Column(
          children: [
            const DefaultTabController(
              length: 3,
              child: TabBar(
                tabs: [
                  Tab(text: 'One'),
                  Tab(text: 'Two'),
                  Tab(text: 'Three'),
                ],
              ),
            ),
            Expanded(
              child: Scrollbar(
                thumbVisibility: true,
                child: ListView(
                  padding: const EdgeInsets.all(16.0),
                  children: [
                    Text('Headline 1', style: Theme.of(context).textTheme.headline1),
                    Text('Headline 2', style: Theme.of(context).textTheme.headline2),
                    Text('Headline 3', style: Theme.of(context).textTheme.headline3),
                    Text('Headline 4', style: Theme.of(context).textTheme.headline4),
                    Text('Subtitle 1', style: Theme.of(context).textTheme.subtitle1),
                    Text('Subtitle 2', style: Theme.of(context).textTheme.subtitle2),
                    Text('Body 1', style: Theme.of(context).textTheme.bodyText1),
                    Text('Body 2', style: Theme.of(context).textTheme.bodyText2),
                    Text('Button', style: Theme.of(context).textTheme.button),
                    Text('Caption', style: Theme.of(context).textTheme.caption),
                    Text('Overline', style: Theme.of(context).textTheme.overline),
                    const Divider(height: 32),
                    ElevatedButton(
                      onPressed: () => {},
                      child: const Text('ElevatedButton'),
                    ),
                    const SizedBox(height: 16),
                    TextButton(
                      onPressed: () => {},
                      child: const Text('TextButton'),
                    ),
                    const SizedBox(height: 16),
                    TextIconButton(
                      onPressed: () => {},
                      child: const Text('TextIconButton'),
                    ),
                    const SizedBox(height: 16),
                    OutlinedButton(
                      onPressed: () => {},
                      child: const Text('OutlinedButton'),
                    ),
                    const SizedBox(height: 16),
                    Align(
                      alignment: AlignmentDirectional.centerStart,
                      child: LinkButton(
                        onPressed: () => {},
                        child: const Text('LinkButton'),
                      ),
                    ),
                    const Divider(height: 32),
                    TextButton(
                      onPressed: () => {
                        ExplanationSheet.show(
                          context,
                          title: 'Title goes here',
                          description: 'Description goes here. This is a demo of the ExplanationSheet!',
                          closeButtonText: 'close',
                        )
                      },
                      child: const Text('Explanation Sheet'),
                    ),
                    TextButton(
                      onPressed: () => {
                        ConfirmActionSheet.show(
                          context,
                          title: 'Title goes here',
                          description: 'Description goes here. This is a demo of the ConfirmActionSheet!',
                          cancelButtonText: 'cancel',
                          confirmButtonText: 'confirm',
                        )
                      },
                      child: const Text('Confirm Action Sheet'),
                    )
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
      bottomNavigationBar: BottomNavigationBar(
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.credit_card),
            label: 'Menu 1',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.qr_code),
            label: 'Menu 2',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.settings_outlined),
            label: 'Menu 3',
          ),
        ],
      ),
    );
  }
}
