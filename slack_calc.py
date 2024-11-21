import math

class BenchmarkTask:
    def __init__(self, period, deadline, priority, name, wcet = 0, slack = 0):
        self.period = period
        self.deadline = deadline
        self.wcet = wcet
        self.priority = priority
        self.name = name
    
    def __str__(self) -> str:
        return self.name + '\'s slack time is: ' + str(self.slack) + ' in percentage: ' + str((self.slack / self.wcet) * 100) + '%'

push_button_server = BenchmarkTask(5000, 100, 7, 'Push button server', 1)
regular_producer = BenchmarkTask(1000, 500, 6, 'Regular producer')
on_call_producer = BenchmarkTask(3000, 800, 4, 'On call producer')
activation_log_reader = BenchmarkTask(3000, 1000, 2, 'Activation log reader')

regular_producer.wcet = int(input('Insert Regular producer WCET: '))
on_call_producer.wcet = int(input('Insert on call producer WCET: '))
activation_log_reader.wcet = int(input('Insert activation log reader WCET: '))
print('\n')

task_list = [push_button_server, regular_producer, on_call_producer, activation_log_reader]

for task in task_list:
    if isinstance(task, BenchmarkTask):
        interference = 0
        for other_task in task_list:
            if isinstance(other_task, BenchmarkTask):
                interference += math.ceil(task.deadline/other_task.period) * other_task.wcet
                if(other_task == task): break
        task.slack = task.deadline - interference
        print(str(task))


