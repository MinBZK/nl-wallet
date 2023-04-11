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
                  padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 32),
                  children: [
                    Text('Headline 1 -> displayLarge', style: Theme.of(context).textTheme.displayLarge),
                    Text('Headline 2 -> displayMedium', style: Theme.of(context).textTheme.displayMedium),
                    Text('Headline 3 -> displaySmall', style: Theme.of(context).textTheme.displaySmall),
                    Text('Headline 4 -> headlineMedium', style: Theme.of(context).textTheme.headlineMedium),
                    Text('Subtitle 1 -> titleMedium', style: Theme.of(context).textTheme.titleMedium),
                    Text('Subtitle 2 -> titleSmall', style: Theme.of(context).textTheme.titleSmall),
                    Text('Body 1 -> bodyLarge', style: Theme.of(context).textTheme.bodyLarge),
                    Text('Body 2 -> bodyMedium', style: Theme.of(context).textTheme.bodyMedium),
                    Text('Button -> labelLarge', style: Theme.of(context).textTheme.labelLarge),
                    Text('Caption -> bodySmall', style: Theme.of(context).textTheme.bodySmall),
                    Text('Overline -> labelSmall', style: Theme.of(context).textTheme.labelSmall),
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
                    ),
                    const Divider(height: 32),
                    Icon(
                      Icons.warning,
                      color: Theme.of(context).colorScheme.error,
                    ),
                    Center(
                      child: Text('Error color',
                          style: Theme.of(context)
                              .textTheme
                              .bodySmall
                              ?.copyWith(color: Theme.of(context).colorScheme.error)),
                    ),
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
