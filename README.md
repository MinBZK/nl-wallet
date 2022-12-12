
# EDI - NL Public Reference Wallet

Within the European Digital Identity program of the Ministry of the Interior and Kingdom Relations, a team is working on a Dutch Public Reference Wallet: the NL Wallet. The program conducts its work in an open and transparent way. This means that the products of the development team – the app designs and source code – are viewable for everyone who wants to. It’s also possible to contribute to the project. We offer three platforms for contributions: [Pleio](https://edi.pleio.nl) as a central hub, [GitHub](https://github.com/MinBZK/nl-wallet-demo-app) for the source code and [Figma](https://www.figma.com/file/dO5pKIIIyDgG0N2ZX4C2xd/2212_V1_Designs_NL-Voorbeeld-Wallet?node-id=1%3A3717&t=nwzAEOJZJEb0eC0o-1) for screens/flows.

The development of a public and open source reference wallet is a dynamic process. What has been released in the current version, may in a next version be developed further, be modified drastically or even removed altogether.

We look forward to your [feedback and great ideas](mailto:edi@minbzk.nl?subject=Contribution%20via%20GitHub). 

### Managing expectations

It’s important to note that a demo app will always be developed ahead of an actually functioning wallet. A clickable demo app contains little technical implementation under the hood. That makes it fast and easy to explore, visualize and discuss different scenarios and possibilities. It also provides a testbed for engineers to explore / experiment / try out certain technical aspects of a wallet.

When you look at the screens in [Figma](https://www.figma.com/file/dO5pKIIIyDgG0N2ZX4C2xd/2212_V1_Designs_NL-Voorbeeld-Wallet?node-id=1%3A3717&t=nwzAEOJZJEb0eC0o-1) or in the demo app, they suggest a lot of functionality that seems to work, but which still requires a lot of development to realize in practice.

It’s possible to use such a wallet in a lot of different situations, for example to share your diploma’s when applying for a job, or to prove that you’re 18+ to buy a beer. With the demo app, we can demonstrate how such a wallet would work in a relatively simple way. 

Although parts of this demo might later be used in wallet related projects, this demo is by no means intended to be production quality code. The goal is to release the first working version of the public reference wallet at the end of 2023 and to make it available in phases for various services.

# Table of contents

- [Current release](#current-release)
- [Documentation](#documentation)
- [Licensing](#licensing)
- [Contributing](#contributing)
- [Getting started](#getting-started)
  * [Setup development environment](#setup-development-environment)
- [File structure](#file-structure)
  * [Code](#code)
  * [Assets](#assets)
  * [Localization](#localization)
- [Architecture](#architecture)
  * [UI Layer](#ui-layer)
  * [Domain Layer](#domain-layer)
  * [Data Layer](#data-layer)
  * [Motivation](#motivation)
- [Conventions](#conventions)
- [Distribution](#distribution)
- [Troubleshooting](#troubleshooting)

# Current release

See the [releases page](https://github.com/MinBZK/nl-wallet-demo-app/releases) for the latest release. The current releases are a clickable demo app that show a number of 'flows' that the user of a wallet can walk through in the context of example use cases. These try out use cases have been chosen because of a variety of reasons: what we want to show, what we want to learn, if they are realistic use cases in practice, and if it is feasible to prototype them in this phase. You can follow the latest work by subscribing to the releases of this GitHub repository at the top of this page.

# Documentation

**TL;DR**

With the NL reference wallet we want to achieve the following things:
- We want to validate the feasibility of the framework as proposed in the EU.
- We want to explore how we can set the bar in terms of privacy protection, security, usability and inclusion.
- We want to learn what this development means for citizens, businesses, other governments and public service providers.
- We want to help citizens, especially those with special needs, in the best way possible.
- We want to offer a testing ground for a variety of use cases.
- We want to share the lessons we learn with the public and share them with the EU.

If you want to learn more about the NL Wallet development, please read the background information on the Pleio hub. The development of the user flows and screens can be followed through [Figma](https://www.figma.com/file/dO5pKIIIyDgG0N2ZX4C2xd/2212_V1_Designs_NL-Voorbeeld-Wallet?node-id=1%3A3717&t=nwzAEOJZJEb0eC0o-1).

# Licensing

The source code of the NL Wallet is released under the [EUPL license](./LICENSES/EUPL-1.2.txt). The documentation is released under the [CC0 license](./LICENSES/CC0-1.0.txt). Please see the [.reuse/dep5](./.reuse/dep5) file for more details, which follows the [Reuse specfication](https://reuse.software/spec/).

# Contributing

We’re releasing the source code with the explicit intention of allowing contributions. The coordination of the project lies with the development team of the European Digital Identity Progam, but we’re open to all contributions. You can directly create a new Pull Request via Github,or contact the community manager via [edi@minbzk.nl](mailto:edi@minbzk.nl?subject=Contribution%20via%20GitHub).

The development team works on the repository in a private fork (for reasons of compliance with existing processes) and shares its work as often as possible. If you watch the repository on GitHub, you will be notified of a new release. We will also send a notification through Pleio.

Although we are open to contributions, please consider the nature of this project as outlined in this Readme. At this stage the most useful way to contribute to the project is to participate on our community site [edi.pleio.nl](https://edi.pleio.nl), and visit our [EDI Meet-ups and/or Heartbeats](https://edi.pleio.nl/events). 

If you plan to make non-trivial changes, we recommend that you open an issue beforehand where we can discuss your planned changes. This increases the chance that we might be able to use your contribution (or it avoids doing work if there are reasons why we wouldn't be able to use it).

Note that all commits should be signed using a GPG key.

# Getting started

## Setup development environment

For help getting started with Flutter development, view the  [online documentation](https://flutter.dev/docs), which offers tutorials, samples, guidance on mobile development, and a full API reference.

# File structure

## Code

All `Dart` code goes in the `lib/` directory and their appropriate sub-directories.

## Assets

All files used by the project (like images, fonts, etc.), go into the `assets/` directory and their appropriate sub-directories.

Note; the `assets/images/` directory contains [resolution-aware images](https://flutter.dev/docs/development/ui/assets-and-images#resolution-aware).

> Copyright note: place all non free (copyrighted) assets used under the `non-free/` directory inside the appropriate asset sub-directory. 

## Localization

Text localization is enabled; currently supporting English & Dutch, with English set as primary (fallback) language. Localized messages are generated based on `ARB` files found in the `lib/src/localization` directory.

To support additional languages, please visit the tutorial on [Internationalizing Flutter apps](https://flutter.dev/docs/development/accessibility-and-localization/internationalization).

Internally, this project uses the commercial Lokalise service to manage translations. This service is currently not accessible for external contributors. For contributors with access, please see [documentation/lokalise.md](./documentation/lokalise.md) for documentation.

# Architecture

The project uses the [flutter_bloc](https://pub.dev/packages/flutter_bloc) package to handle state management.

On top of that it follows the [BLoC Architecture](https://bloclibrary.dev/#/architecture) guide, with a slightly fleshed out domain layer.

This architecture relies on three conceptual layers, namely:
- [UI Layer](#ui-layer)
- [Domain Layer](#domain-layer)
- [Data Layer](#data-layer)

![Architecture Overview](./architecture_overview.png)

## UI Layer

Responsible for displaying the application data on screen.

In the above diagram the **UI** node likely represents a `Widget`, e.g. in the form of a `xyzScreen`, that observes the Bloc using one of the `flutter_bloc` provided Widgets. E.g. `BlocBuilder`.

When the user interacts with the UI, the UI is responsible for sending a corresponding `Event` to the associated BLoC. The BLoC than processes this event and emits an updated `State` to the UI, causing the UI to rebuild and render the new state.

## Domain Layer

Encapsulate business logic to make it reusable. UseCases are likely to be used by BLoCs to interact with data, allowing the BLoCs to be concise and keep their focus on converting events into new states.

Naming convention `verb in present tense + noun/what (optional) + UseCase` e.g. LogoutUserUseCase.

## Data Layer

Exposes application data to the rest of the application. Responsible for any CRUD like operations or network interactions,  likely through other classes in the form of DataSources.

## Motivation

The reason we opted for this BLoC layered approach with the intermediary domain layer is to optimize for: Re-usability, Testability and Readability.

**Re-usable** because the usecases in the domain layer are focused on a single task, making them convenient to re-use in multiple blocs when the same data or interaction is required on multiple (ui) screens.

**Testable** because with the abstraction to other layers in the form of dependencies of a class, the dependencies can be easily swapped out by mock implementations, allowing us to create small, non-flaky unit tests of all individual  components.

**Readable** because with there is a clear separation of concerns between the layers, the UI is driven by data models (not by state living in UI components) and can thus be easily inspected, there is a single source of truth for the data  in the data layer and there is a unidirectional data flow in the ui layer making it easier to reason about the transitions between different states.

Finally, since while we are developing the initial Proof of Concept it is unlikely that we will be working with real datasources, this abstraction allows us to get started now, and in theory quickly migrate to a fully functional app (once the data comes online) by replacing our MockRepositories / MockDataSources with the actual implementations, without touching anything in the Domain or UI Layer.


# Conventions

## Git

### Commit message
- Capitalize the subject line
- Use the imperative mood in the subject line
- Do not end the subject line with a period
- Wrap lines at 72 characters

### PR title
See [commit message](#commit-message).

### PR merge
- Default to squash merge (combined with PR title conventions)

## Dart
* Max. line length is set to 120 (Dart defaults to 80)
* Relative imports for all project files below `src` folder; for example: `import 'bloc/wallet_bloc.dart';`
* Trailing commas are added by default; unless it compromises readability

## Naming
* Folder naming is `singular` for folders below `src`; for example: `src/feature/wallet/widget/...`

# Distribution

Follow these steps to (force) distribute internal `alpha` & `beta` builds that target the Android platform":

### Alpha

> Use `Alpha` distribution at any time during development cycle.

* Push commit of your choosing to: alpha/v{X.Y.Z}
* After the GitHub Action has completed successfully; install the release via F-Droid repo

### Beta

> Use `Beta` distribution at the end of a sprint cycle; to represent the sprint demo version.

* `$ git fetch && git pull`
* Push `main` branch to: beta/v{X.Y.Z}
* After the GitHub Action has completed successfully; install the release via F-Droid repo
 
# Troubleshooting

### Initial checkout / branch switch
Generate/update localisation files (to compile/run the project successfully):

    $ flutter gen-l10n
