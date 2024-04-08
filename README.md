<img src="assets/Logo.svg" alt="drawing" width="400"/>

A simple viewer and simulator for task charts.

## Tutorial

### User Interface
tsyncs offers a convinient way to load and visualize task charts. At the top you find the menu bar. Here you can import and export task charts, save and load your current session and start a new session. You can also edit the graph and change the View.
![UserInterface](assets/UI-Description.png)

### File Menu
To load a task select `File -> Import Graph...` and select a CSV file containing the task chart. You can also export the current graph as CSV file by selecting `File -> Export Graph...`.
You can also Save the current session by selecting `File -> Save Graph...` and load a saved session by selecting `File -> Load Graph`.

To start a new, empty session select `File -> New Graph`.

### Editing the Graph
The task chart will be displayed in the main window. You can zoom in and out using the mouse wheel and pan by dragging the mouse. You can also move the tasks by dragging them.

The tasks and mutexes are connected by arrows, which represent the dependencies between the tasks. The arrows are colored based on where the tasks are flowing. A runing task is highlighted by a green border, a waiting task is highlighted with a red boarder.

You can change the duration of a task by changing the value in the upper right corner of the task. Below that you find the time the task remains active. You can change both values by clicking on them and typing in the new value or dragging your mouse or finger left and right. You can also change the priority of a task by clicking on the priority value on the lower right corner and typing in the new value or dragging your mouse or finger left and right.

If you want to change the value of a mutex by clicking in the middle of the mutex and typing in the new value or dragging your mouse or finger left and right.

#### Adding Tasks and Mutexes
To add a new task right click at an empty space in the canvas. This will add a new task. If you right click on a task you can connect it to existing mutexes by clicking on the mutex you want to connect it to. If you click on a task a mutex will be created between the two tasks.

#### Deleting a Task or Mutex
To deleate a task or mutex click go to `Edit -> Delete Mode` to enable the delete mode. Now you can click on a task or mutex to delete it.
To exit the delete mode click on the Warning that appears if the deleate mode is active.

### Simulation Settings
At the bottom you find the simulation settings. On the left you can change the simulation speed by using the slider in the `ticks per second` bottom left corner. The simulation speed represents the number of ticks per second. One tick will reduce the remaining time of the active task by one.

You can also pause the animation by clicking the Pause ⏸ button. If the animation is paused, you can run a Single Tick with the `Single Step` button. This button scheddules a new Tick to run which can be seen and eddited in the `ticks remaining` filed next  a text field with the remaining ticks will be displayed. You can change the remaining ticks by clicking on the text field and typing in the new value or dragging your mouse or finger left and right. You can also run a single tick by clicking the `Single Step` button. By clicking Play ▶️ you can continue the automatic simulation.

### File format
You can Import and Export your task as CSV files. The CSV file has the following header:
```csv
Type; ID; Parameters
```
There are two types for entries in the CSV file `Task` and `Mutex`.

Task entries take the following format:
```csv
Task; Position X; Position Y; ID; Task-Name; Activity-Name; Priority; Duration; Remaining Duration; [Semicolon seperated list of Connected Mutex IDs]
```

Mutex entries take the following format:
```csv
Mutex; Position X; Position Y; ID; Mutex Value; [Semicolon seperated list of Connected Task IDs]
```

#### Example CSV file
```csv
Type;Position X;Position Y;ID;Parameters...
"Task";Position X;Position Y;ID;Task Name;Activity Name;Priority;Duration;Remaining Duration;[Semicolon seperated list of Connected Mutex IDs]
Task;300;100;0;Task 2;Activity 2;0;3;0;0;2
Task;150;250;1;Task 1;Activity 1;0;3;0;1;4
Task;150;400;2;Task 5;Activity 5b;0;1;0;7;8
Task;450;400;3;Task 5;Activity 5a;0;2;0;9
Task;450;250;4;Task 3;Activity 3;0;2;0;2;5
Task;600;100;5;Task 4;Activity 4;0;3;0;2;3
Task;750;250;6;Task 6;Activity 6;0;3;0;6
"Mutex";Position X;Position Y;ID;Mutex Value;[Semicolon seperated list of Connected Task IDs]
Mutex;225;175;1;0;0
Mutex;300;380;9;0;2
Mutex;150;325;7;0;1
Mutex;300;250;4;0;4
Mutex;600;325;6;0;3
Mutex;450;100;0;0;5
Mutex;450;150;2;1;4;5;0
Mutex;300;420;8;1;3
Mutex;600;250;5;0;6
Mutex;675;175;3;0;6
```
