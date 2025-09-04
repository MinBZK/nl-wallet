# About the Functional Design
The functional design (FD) describes the complete functional behaviour of the system, including every interaction with its users and external systems. It contains the following components:
* **Use Cases (UC)** describes a case in which a user uses the system to achieve some goal.
* **Partial Flows (PF)** describes a part of the interaction used in two or more use cases.
* **Logical Test Cases (LTC)** describe the high level tests of the functionality. 

## Use Case breakdown
The user-system interaction is broken down into multiple use cases. Every use case should have a clear start and end and provide some value to the user. This breakdown is done in such a way to minimize overlap between use cases, i.e. to avoid duplicate specification of the same flow. This leads to a web of use cases that are connected to on another, with some smaller use cases that add seemingly little value (e.g. 'open the app', 'open the menu').  

When several use cases repeat the same steps (e.g. Confirm a protected action with PIN) we use a Partial Flow to describe that flow once, and refer to it from the Use Cases that use it.

## Use Cases (UC)
A use case is specified by a single document (`UC[nr]_[name].md`) which contains the following:
* An **overview** of its key aspects (including summary, goal and more);
* A **flow description** describing the interaction step by step.

### Use Case overview
The UC overview contains a table with the following elements:
| Aspect                       | Description                                                                                                                                            |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Summary**                  | A brief summary of the interaction between user and system, including at least the primary scenario and possibly some important alternative scenarios. |
| **Goal**                     | The goal the user wishes to achieve by executing this use case.                                                                                        |
| **Preconditions**            | A list of all conditions that must satisfied in order for the UC to complete successfully. (not including triggers).                                   |
| **Postconditions**           | A list of all conditions that will be satisfied after the UC completed successfully, i.e. the important consequences.                                  |
| **Triggered by**             | A list of all triggers that start this use case (e.g. selecting a particular menu-item, completing another use case).                                  |
| **Additional Documentation** | A list of all other (technical) documentation relevant to this use case.                                                                               |
| **Logical Test Cases**         | A list of logical testcases that cover the functionality of this use case.                                                                               |
| **Possible errors**          | A list of all possible expected errors that may be encountered when executing this use case. This does not include unexpected (programming) errors.    |

### Use Case flow description
The UC flow description describes the complete interaction between user and system. It includes every possible user action, every system response, important system decision points and processing actions.

The flow description is in table form and looks something like this:
| #       | Description                                                                                     | Next | 
| ------- | ----------------------------------------------------------------------------------------------- | ---- |
| **1**   | **PRIMARY SCENARIO**                                                                            |      | 
| **1.1** | **System displays screen 'Home'**<ul><li>Message: hello world.</li><li>Actions: Exit.</li></ul> |      |
| 1.1a    | User selects Exit                                                                               | 1.2  |
| ...     | ...                                                                                             | ...  |

The table includes three types of entries (matching the example above):
* **Scenarios**: the primary scenario (happy flow) and all alternative scenarios and error scenarios;
* **Nodes**: all screens, prompts, decision points and important processing steps;
* **Edges**: all user actions, decision cases and errors that connect the nodes.

#### Scenarios
All scenarios part of the use case are included. They describe one of the following:
1. "PRIMARY SCENARIO" (always first, #1);
2. An alternative scenario phrased as a user activity (e.g. "GET HELP", "READ PRIVACY STATEMENT");
3. An alternative scenario phrased as a situation starting with "WHEN".

Note: error scenarios will be described in a future version.

#### Nodes
The nodes represent the key steps in the user-system interaction. Possible nodes:
1. "System displays screen '{screen name}'"
2. "System displays prompt '{prompt name}'"
3. "System determines {condition on state/input/environment}"
4. "System validates {input and constraint}"
5. "System executes {important processing task}"
6. "Operating system {action}"

Processing steps are included when they are either time consuming (user must wait), fallible (errors are expected), interactions with external systems or have important consequences.

Subsequent processing steps or decision nodes may be merged into a single node if it contributes to the legibility of the table.

##### Node details
Following the node title is a list of details. Details to include when applicable:
1. "Message: [...]" summarizing the key message of a screen or prompt. (not the literal text)
2. "Inputs: [...]" input elements presented to the user (except for buttons/links).
3. "Actions: [...]" a list of actions presented to the user. (not the literal text)
4. "Contents: [...]" a list of other important content elements presented to the user.
5. "Duration: X to Y seconds" summarizing the expected time period users may have to wait for the system.

Purely graphical details are left to the graphical design. Text literals are left to the system's language configuration.

#### Edges
The edges describe either actions by the user, events or outcomes of system actions. They may include:
1. "User [verb] [...]"
2. "Case: [predicate]"
3. "Error: [error situation]"
4. "Event: [event description]"

An edge may:
1. have a side-effect (in the 'description' column, on a new line starting with "&rarr;", e.g. "&rarr; Flashlight is enabled");
2. point to another node (by referencing its number the 'next' column);
3. point to another use case (by referencing it in 'next': "Go to: UC...")
4. point to a separate error flow description (by referencing it in 'next'; "Error Flow: [error name]")
5. specify "Back" to denote that the user returns to the previous screen they visisted.

## Partial Flows (PF)
In order to avoid duplication in the FD we sometimes use partial flows (PFs). Partial flows are a part of a use case flow that occurs in two or more use cases and is big enough to justify reuse. A PF is similar to a use case with the following key differences:
* The PF Overview is like the UC Overview, but it does not include "Triggered by"
* The PF Overview includes "Used in" which references all UCs that use the PF.
* The PF Overview includes "Parameters" which describe the parameters that allow the UCs to customize the PF.
* The PF Flow description is like the UC Flow description.
* When the PF returns flow to the UC, the 'next' contains "Return: [return value]"

## Logical Test Cases (LTC)
These define high-level, black-box tests that verify the functional behavior described by the Use Cases (UC) and Partial Flows (PF).
Each UC and RF references the relevant LTC ID and title in the LTC field to ensure traceability and coverage.

Format: Gherkin-style Given / When / Then (/ And) steps.
Scope: Functional behavior as observable by the user or external systems. Non-functional tests (performance, accessibility, etc.) are out of scope here.

## For authors
When writing / editing the FD, please take care of the following:
1. Keep names and descriptions as short as possible, but not shorter
2. Consistent formatting
3. Consistent grammar
4. When referencing other use cases, always use its full name. Also link to the file.
