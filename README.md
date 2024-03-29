<img src="assets/Logo.svg" alt="drawing" width="400"/>

A simple viewer and simulator for task charts.

## Tutorial
tsyncs offers a convinient way to load and visualize task charts. To load a task select `File -> Open` and select a CSV file containing the task chart. The task chart will be displayed in the main window. You can zoom in and out using the mouse wheel and pan by dragging the mouse. You can also move the tasks by dragging them.

The tasks and mutexes are connected by arrows, which represent the dependencies between the tasks. The arrows are colored based on where the tasks are flowing. A runing task is highlighted by a green border, a waiting task is highlighted with a red boarder.

You can change the duration of a task by changing the value in the upper right corner of the task. Below that you find the time the task remains active. You can change both values by clicking on them and typing in the new value or dragging your mouse or finger left and right. You can also change the priority of a task by clicking on the priority value on the lower right corner and typing in the new value or dragging your mouse or finger left and right.

If you want to change the value of a mutex by clicking in the middle of the mutex and typing in the new value or dragging your mouse or finger left and right.

You can change the simulation speed by using the slider in the `Animation Speed` bottom right corner. The simulation speed represents the number of ticks per second. One tick will reduce the remaining time of the active task by one.

You can also pause the animation by clicking the `Pause` button. if the animation is paused, a text field with the remaining ticks will be displayed. You can change the remaining ticks by clicking on the text field and typing in the new value or dragging your mouse or finger left and right. You can also run a single tick by clicking the `Single Step` button.

### File format
You can Import and Export your task as CSV files. The CSV file has the following header:
```csv
Type; ID; Parameters
```
There are two types for entries in the CSV file `Task` and `Mutex`.

Task entries take the following format:
```csv
Task; ID; Task-Name; Activity-Name; Duration, Priority, [Comma seperated list of Connected Mutex IDs]
```

Mutex entries take the following format:
```csv
Mutex; ID; Value; [Comma seperated list of Connected Task IDs]
```

#### Example CSV file
```csv
Type,ID,Parameters
Task;0;Task 2;Activiy 2;3;0;0;2
Task;1;Task 1;Activiy 1;3;0;4;1
Task;2;Task 5;Activiy 5b;1;0;8;7
Task;3;Task 5;Activiy 5a;2;0;9
Task;4;Task 3;Activiy 3;2;2;5;2
Task;5;Task 4;Activiy 4;3;0;3;2
Task;6;Task 6;Activiy 6;3;0;6
Mutex;1;0;0
Mutex;3;0;6
Mutex;6;0;3
Mutex;4;0;4
Mutex;2;0;4;5;0
Mutex;9;0;2
Mutex;8;0;3
Mutex;7;0;1
Mutex;0;0;5
Mutex;5;0;6
```