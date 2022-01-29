changelog for CakeRabbit
=======



### Version 0.1.2

- [x] When external service just start, too fast to check the service status problem, set the service interval check time
- [x] when the service in the register center,  the loop register thread should not exit, but when the service access run, reload the service from persistence, this will fix the problem
- [ ] if the consul connection time out, retry to connect later, not exit the tread at once.
