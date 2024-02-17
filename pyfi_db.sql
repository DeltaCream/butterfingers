CREATE DATABASE pyfi_db;
USE pyfi_db;

CREATE TABLE employee_role(
	role_code SMALLINT UNSIGNED NOT NULL,
    role_name varchar(125) NOT NULL,
    PRIMARY KEY(role_code)
);
INSERT INTO employee_role VALUES(1,'Management');
INSERT INTO employee_role VALUES(2,'Production/Office Staff');

CREATE TABLE employee(
	emp_id BIGINT UNSIGNED NOT NULL,
    fname varchar(50) NOT NULL,
    mname varchar(50),
    lname varchar(50) NOT NULL,
    dob date NOT NULL,
    doh date NOT NULL,
    role_code SMALLINT UNSIGNED NOT NULL,
    tin_num INT UNSIGNED NOT NULL,
    image blob,
    PRIMARY KEY(emp_id),
    foreign key(role_code) references employee_role(role_code)
);

CREATE TABLE employee_status(
	emp_id BIGINT UNSIGNED NOT NULL,
    isActive boolean NOT NULL default TRUE,
    days_tenured BIGINT UNSIGNED NOT NULL DEFAULT 1,
    PRIMARY KEY(emp_id),
    foreign key(emp_id) references employee(emp_id)
);

CREATE TABLE employee_leave_status(
	emp_id BIGINT UNSIGNED NOT NULL,
    onLeave boolean NOT NULL default FALSE,
    paid_leaves SMALLINT UNSIGNED NOT NULL DEFAULT 5,
    PRIMARY KEY(emp_id),
    foreign key(emp_id) references employee_status(emp_id)
);

CREATE TABLE managerial_positions(
	managerial_position_code SMALLINT UNSIGNED NOT NULL,
	position_name VARCHAR(125) NOT NULL,
    PRIMARY KEY(managerial_position_code)
);
INSERT INTO managerial_positions VALUES(1, 'General Manager');
INSERT INTO managerial_positions VALUES(2, 'Human Resources');

CREATE TABLE management(
	emp_id BIGINT UNSIGNED NOT NULL,
    managerial_position_code SMALLINT UNSIGNED NOT NULL,
	PRIMARY KEY(emp_id),
    foreign key(emp_id) references employee(emp_id),
	foreign key(managerial_position_code) references managerial_positions(managerial_position_code)
);

CREATE TABLE user_accounts(
	emp_id BIGINT UNSIGNED NOT NULL,
    username varchar(255) UNIQUE NOT NULL,
    user_password varchar(255) NOT NULL,
    PRIMARY KEY(emp_id),
    foreign key(emp_id) references management(emp_id)
);

CREATE TABLE staff_positions(
	position_code SMALLINT UNSIGNED NOT NULL,
    position_name varchar(255) NOT NULL,
    PRIMARY KEY(position_code)
);

CREATE TABLE production_staff(
	emp_id BIGINT UNSIGNED NOT NULL,
    position_code SMALLINT UNSIGNED NOT NULL,
	PRIMARY KEY(emp_id),
    foreign key(emp_id) references employee(emp_id),
	foreign key(position_code) references staff_positions(position_code)
);

CREATE TABLE enrolled_fingerprints(
	emp_id BIGINT UNSIGNED NOT NULL,
    fprint_uuid varchar(36) NOT NULL UNIQUE,
    PRIMARY KEY(emp_id),
    foreign key(emp_id) references production_staff(emp_id)
); 

CREATE TABLE attendance_status(
	attendance_status_code SMALLINT UNSIGNED NOT NULL,
    attendance_status_meaning VARCHAR(15) NOT NULL,
    PRIMARY KEY(attendance_status_code)
);
INSERT INTO attendance_status VALUES(1, 'Entry');
INSERT INTO attendance_status VALUES(2, 'Exit');
INSERT INTO attendance_status VALUES(3, 'Absent');
INSERT INTO attendance_status VALUES(4, 'On Leave');

CREATE TABLE attendance_records(
	emp_id BIGINT UNSIGNED NOT NULL,
    attendance_date DATE NOT NULL,
    attendance_time TIME NOT NULL,
    attendance_status_code SMALLINT UNSIGNED NOT NULL,
    PRIMARY KEY(emp_id),
    foreign key(emp_id) references production_staff(emp_id),
    foreign key(attendance_status_code) references attendance_status(attendance_status_code)
);
DELIMITER //
Create Procedure add_emp_role(IN empid BIGINT UNSIGNED, IN emp_position SMALLINT UNSIGNED)
	Begin
		if((select role_code from employee WHERE emp_id = empid) = 1) then
			INSERT INTO management VALUES(empid, emp_position);
        elseif ((select role_code from employee WHERE emp_id = empid) = 2) then
			INSERT INTO production_staff VALUES(empid, emp_position);
		end if;
    End
DELIMITER ;

DELIMITER //
Create Procedure add_mng_user_acc(IN empid BIGINT UNSIGNED, IN uname varchar(255),  IN pwd varchar(255))
	Begin
		INSERT INTO user_accounts VALUES (empid, uname, pwd);
    End
DELIMITER ;

Create Procedure enumerate_unenrolled_employees()
Begin
	select production_staff.emp_id As "Employee ID", employee.fname As "First Name",employee.lname As "Last Name" from production_staff join employee using(emp_id) where production_staff.emp_id not in (select emp_id from enrolled_fingerprints);
End
DELIMITER ; 

Create Procedure save_fprint_identifier(IN empid BIGINT UNSIGNED, IN fprint_id varchar(36))
Begin
	insert into enrolled_fingerprints values (empid, fprint_id); 
End
DELIMITER ;

DELIMITER //
Create Procedure record_attendance(IN uuid varchar(36))
	Begin
		IF(EXISTS(SELECT emp_id, fprint_uuid from enrolled_fingerprints where fprint_uuid = uuid)) then
		    SET @emp_id = (SELECT emp_id from enrolled_fingerprints where fprint_uuid = uuid);
			SET @last_attendance_code = (SELECT attendance_status_code from attendance_records where emp_id = @emp_id and attendance_date = DATE(NOW()) ORDER BY attendance_date DESC LIMIT 1);
            IF(@last_attendance_code = 2) then
				insert into attendance_records VALUES(@emp_id, DATE(NOW()), TIME(NOW()), 1);
			elseif(@last_attendance_code = 1) then
				insert into attendance_records VALUES(@emp_id, DATE(NOW()), TIME(NOW()), 2);
			else
				insert into attendance_records VALUES(@emp_id, DATE(NOW()), TIME(NOW()), 1);
			end if;
		else 
			SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'No such fingerprint is enrolled in the Database';
        end if;
    End
DELIMITER ;
