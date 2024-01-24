CREATE DATABASE pyfi_db;
USE pyfi_db;

CREATE TABLE employee_role(
	role_code SMALLINT UNSIGNED NOT NULL,
    role_name varchar(125) NOT NULL,
    PRIMARY KEY(role_code)
);

CREATE TABLE employee(
	emp_id BIGINT UNSIGNED NOT NULL,
    fname varchar(50) NOT NULL,
    mname varchar(50),
    lname varchar(50) NOT NULL,
    dob date NOT NULL,
    doh boolean NOT NULL,
    role_code SMALLINT UNSIGNED NOT NULL,
    tin_num INT UNSIGNED NOT NULL,
    image blob NOT NULL,
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
    password varchar(255) UNIQUE NOT NULL,
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

CREATE TABLE attendance_records(
	emp_id BIGINT UNSIGNED NOT NULL,
    attendance_date DATE NOT NULL,
    attendance_time TIME NOT NULL,
    attendance_status_code SMALLINT UNSIGNED NOT NULL,
    PRIMARY KEY(emp_id),
    foreign key(emp_id) references production_staff(emp_id),
    foreign key(attendance_status_code) references attendance_status(attendance_status_code)
);




