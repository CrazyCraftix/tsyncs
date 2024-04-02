<img src="assets/Logo.svg" alt="drawing" width="400"/>

A simple viewer and simulator for task charts.

## Tutorial
tsyncs offers a convinient way to load and visualize task charts. To load a task select `File -> Import Graph...` and select a CSV file containing the task chart. You can also export the current graph as CSV file by selecting `File -> Export Graph...` The task chart will be displayed in the main window. You can zoom in and out using the mouse wheel and pan by dragging the mouse. You can also move the tasks by dragging them.

The tasks and mutexes are connected by arrows, which represent the dependencies between the tasks. The arrows are colored based on where the tasks are flowing. A runing task is highlighted by a green border, a waiting task is highlighted with a red boarder.

You can change the duration of a task by changing the value in the upper right corner of the task. Below that you find the time the task remains active. You can change both values by clicking on them and typing in the new value or dragging your mouse or finger left and right. You can also change the priority of a task by clicking on the priority value on the lower right corner and typing in the new value or dragging your mouse or finger left and right.

If you want to change the value of a mutex by clicking in the middle of the mutex and typing in the new value or dragging your mouse or finger left and right.

You can change the simulation speed by using the slider in the `Ticks per Second` bottom right corner. The simulation speed represents the number of ticks per second. One tick will reduce the remaining time of the active task by one.

You can also pause the animation by clicking the Pause ⏸ button. If the animation is paused, a text field with the remaining ticks will be displayed. You can change the remaining ticks by clicking on the text field and typing in the new value or dragging your mouse or finger left and right. You can also run a single tick by clicking the `Single Step` button. By clicking Play ▶️ you can continue the automatic simulation.
### File format
You can Import and Export your task as CSV files. The CSV file has the following header:
```csv
Type; ID; Parameters
```
There are two types for entries in the CSV file `Task` and `Mutex`.

Task entries take the following format:
```csv
Task; Position X; Position Y; ID; Task-Name; Activity-Name; Duration, Priority, [Comma seperated list of Connected Mutex IDs]
```

Mutex entries take the following format:
```csv
Mutex; Position X; Position Y; ID; Value; [Comma seperated list of Connected Task IDs]
```

#### Example CSV file
```csv
Type;Position X;Position Y;ID;Parameters
Task;300;100;0;Task 2;Activiy 2;3;0;0;2
Task;150;250;1;Task 1;Activiy 1;3;0;1;4
Task;150;400;2;Task 5;Activiy 5b;1;0;8;7
Task;450;400;3;Task 5;Activiy 5a;2;0;9
Task;450;250;4;Task 3;Activiy 3;2;0;2;5
Task;600;100;5;Task 4;Activiy 4;3;0;2;3
Task;750;250;6;Task 6;Activiy 6;3;0;6
Mutex;600;250;5;0;6
Mutex;300;420;8;1;3
Mutex;600;325;6;0;3
Mutex;450;100;0;0;5
Mutex;300;250;4;1;4
Mutex;300;380;9;0;2
Mutex;450;150;2;1;5;0;4
Mutex;225;175;1;1;0
Mutex;675;175;3;0;6
Mutex;150;325;7;0;1
```