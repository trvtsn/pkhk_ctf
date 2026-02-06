-- MySQL Workbench Forward Engineering

SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0;
SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0;
SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION';

-- -----------------------------------------------------
-- Schema mydb
-- -----------------------------------------------------
-- -----------------------------------------------------
-- Schema ctfpkhk
-- -----------------------------------------------------

-- -----------------------------------------------------
-- Schema ctfpkhk
-- -----------------------------------------------------
CREATE SCHEMA IF NOT EXISTS `ctfpkhk` DEFAULT CHARACTER SET utf8mb3 ;
USE `ctfpkhk` ;

-- -----------------------------------------------------
-- Table `ctfpkhk`.`events`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`events` (
  `id` CHAR(36) NOT NULL,
  `name` VARCHAR(45) NOT NULL,
  `description` TEXT NULL DEFAULT NULL,
  `start_at` TIMESTAMP NOT NULL,
  `end_at` TIMESTAMP NOT NULL,
  `visible_to_groups` VARCHAR(50) NOT NULL,
  PRIMARY KEY (`id`))
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`challenges`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`challenges` (
  `id` CHAR(36) NOT NULL,
  `event_id` CHAR(36) NOT NULL,
  `name` VARCHAR(45) NOT NULL,
  `description` TEXT NULL DEFAULT NULL,
  `category` VARCHAR(45) NULL DEFAULT NULL,
  `difficulty` TINYINT NOT NULL,
  `points` INT UNSIGNED NOT NULL,
  `flag_hash` VARCHAR(100) NOT NULL,
  `visible_to_groups` VARCHAR(50) NOT NULL,
  `vm_id` VARCHAR(45) NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `fk_challenges_events1`
    FOREIGN KEY (`event_id`)
    REFERENCES `ctfpkhk`.`events` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE)
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;

CREATE INDEX `fk_challenges_events1_idx` ON `ctfpkhk`.`challenges` (`event_id` ASC) VISIBLE;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`users`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`users` (
  `id` CHAR(36) NOT NULL,
  `username` VARCHAR(40) NOT NULL,
  `email` VARCHAR(90) NOT NULL,
  `pw_hash` VARCHAR(100) NOT NULL,
  `created_at` TIMESTAMP NOT NULL,
  `last_active_at` TIMESTAMP NOT NULL,
  `role` VARCHAR(14) NOT NULL,
  `points` INT UNSIGNED NOT NULL,
  `group` VARCHAR(30) NOT NULL,
  `auth_type` VARCHAR(10) NOT NULL,
  PRIMARY KEY (`id`))
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;

CREATE UNIQUE INDEX `username_UNIQUE` ON `ctfpkhk`.`users` (`username` ASC) VISIBLE;

CREATE INDEX `email_idx` ON `ctfpkhk`.`users` (`email` ASC) INVISIBLE;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`attachments`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`attachments` (
  `id` CHAR(36) NOT NULL,
  `challenge_id` CHAR(36) NULL DEFAULT NULL,
  `event_id` CHAR(36) NULL DEFAULT NULL,
  `user_id` CHAR(36) NULL DEFAULT NULL,
  `file_name` VARCHAR(90) NOT NULL,
  `file_blob` MEDIUMBLOB NOT NULL,
  `file_type` VARCHAR(20) NOT NULL,
  `mime_type` VARCHAR(45) NULL DEFAULT NULL,
  `file_size` INT GENERATED ALWAYS AS (length(`file_blob`)) VIRTUAL,
  PRIMARY KEY (`id`),
  CONSTRAINT `fk_attachments_challenges1`
    FOREIGN KEY (`challenge_id`)
    REFERENCES `ctfpkhk`.`challenges` (`id`),
  CONSTRAINT `fk_attachments_events1`
    FOREIGN KEY (`event_id`)
    REFERENCES `ctfpkhk`.`events` (`id`),
  CONSTRAINT `fk_attachments_users1`
    FOREIGN KEY (`user_id`)
    REFERENCES `ctfpkhk`.`users` (`id`))
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;

CREATE UNIQUE INDEX `user_id_UNIQUE` ON `ctfpkhk`.`attachments` (`user_id` ASC) VISIBLE;

CREATE INDEX `fk_attachments_users1_idx` ON `ctfpkhk`.`attachments` (`user_id` ASC) VISIBLE;

CREATE INDEX `fk_attachments_events1_idx` ON `ctfpkhk`.`attachments` (`event_id` ASC) VISIBLE;

CREATE INDEX `fk_attachments_challenges1_idx` ON `ctfpkhk`.`attachments` (`challenge_id` ASC) VISIBLE;

CREATE INDEX `file_name_idx` ON `ctfpkhk`.`attachments` (`file_name` ASC) VISIBLE;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`ldap`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`ldap` (
  `restriction` ENUM('') NOT NULL,
  `url` VARCHAR(100) NOT NULL,
  `bind_dn` VARCHAR(100) NOT NULL,
  `bind_pw` VARCHAR(64) NOT NULL,
  `base_dn` VARCHAR(100) NOT NULL,
  `enabled` TINYINT NOT NULL,
  PRIMARY KEY (`restriction`))
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`proxmox`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`proxmox` (
  `restriction` ENUM('') NOT NULL,
  `base_url` VARCHAR(100) NOT NULL,
  `api_path` VARCHAR(100) NOT NULL,
  `node` VARCHAR(45) NOT NULL,
  `username` VARCHAR(64) NULL DEFAULT NULL,
  `password` VARCHAR(64) NULL DEFAULT NULL,
  `api_token` VARCHAR(128) NULL DEFAULT NULL,
  `auth_type` VARCHAR(10) NOT NULL,
  PRIMARY KEY (`restriction`))
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`proxmox_instances`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`proxmox_instances` (
  `id` CHAR(36) NOT NULL,
  `challenge_id` CHAR(36) NOT NULL,
  `user_id` CHAR(36) NOT NULL,
  `vm_id` INT NOT NULL,
  `created_at` TIMESTAMP NOT NULL,
  `end_at` TIMESTAMP NOT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `fk_proxmox_instances_challenges1`
    FOREIGN KEY (`challenge_id`)
    REFERENCES `ctfpkhk`.`challenges` (`id`),
  CONSTRAINT `fk_proxmox_instances_users1`
    FOREIGN KEY (`user_id`)
    REFERENCES `ctfpkhk`.`users` (`id`))
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;

CREATE UNIQUE INDEX `vm_id_UNIQUE` ON `ctfpkhk`.`proxmox_instances` (`vm_id` ASC) VISIBLE;

CREATE INDEX `fk_proxmox_instances_challenges1_idx` ON `ctfpkhk`.`proxmox_instances` (`challenge_id` ASC) VISIBLE;

CREATE INDEX `fk_proxmox_instances_users1_idx` ON `ctfpkhk`.`proxmox_instances` (`user_id` ASC) VISIBLE;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`submissions`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`submissions` (
  `id` CHAR(36) NOT NULL,
  `challenge_id` CHAR(36) NOT NULL,
  `event_id` CHAR(36) NOT NULL,
  `user_id` CHAR(36) NOT NULL,
  `points` INT UNSIGNED NOT NULL,
  `solved_at` TIMESTAMP NOT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `fk_submissions_challenges1`
    FOREIGN KEY (`challenge_id`)
    REFERENCES `ctfpkhk`.`challenges` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
  CONSTRAINT `fk_submissions_events1`
    FOREIGN KEY (`event_id`)
    REFERENCES `ctfpkhk`.`events` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
  CONSTRAINT `fk_submissions_users1`
    FOREIGN KEY (`user_id`)
    REFERENCES `ctfpkhk`.`users` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE)
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;

CREATE INDEX `fk_leaderboard_users1_idx` ON `ctfpkhk`.`submissions` (`user_id` ASC) VISIBLE;

CREATE INDEX `fk_leaderboard_events1_idx` ON `ctfpkhk`.`submissions` (`event_id` ASC) VISIBLE;

CREATE INDEX `fk_leaderboard_challenges1_idx` ON `ctfpkhk`.`submissions` (`challenge_id` ASC) VISIBLE;

USE `ctfpkhk`;

DELIMITER $$
USE `ctfpkhk`$$
CREATE
DEFINER=`root`@`localhost`
TRIGGER `ctfpkhk`.`submissions_AFTER_DELETE`
AFTER DELETE ON `ctfpkhk`.`submissions`
FOR EACH ROW
BEGIN
	UPDATE users
    SET points = 
    GREATEST(COALESCE(points,0) -
    COALESCE(OLD.points,0), 0)
    WHERE id = OLD.user_id;
END$$

USE `ctfpkhk`$$
CREATE
DEFINER=`root`@`localhost`
TRIGGER `ctfpkhk`.`submissions_AFTER_INSERT`
AFTER INSERT ON `ctfpkhk`.`submissions`
FOR EACH ROW
BEGIN
	UPDATE users
    SET points = 
    COALESCE(points,0) +
    COALESCE(NEW.points,0)
    WHERE id = NEW.user_id;
END$$

USE `ctfpkhk`$$
CREATE
DEFINER=`root`@`localhost`
TRIGGER `ctfpkhk`.`submissions_AFTER_UPDATE`
AFTER UPDATE ON `ctfpkhk`.`submissions`
FOR EACH ROW
BEGIN
	IF OLD.user_id = NEW.user_id
	THEN
		UPDATE users
		SET points = 
		GREATEST(COALESCE(points,0) + 
		COALESCE(NEW.points,0) - 
		COALESCE(OLD.points,0), 0)
		WHERE id = NEW.user_id;
	ELSE
		UPDATE users
		SET points = 
		GREATEST(COALESCE(points,0) - 
		COALESCE(OLD.points,0), 0)
		WHERE id = OLD.user_id;
        
		UPDATE users
		SET points = 
		COALESCE(points,0) + 
		COALESCE(NEW.points,0) 
		WHERE id = NEW.user_id;
	END IF;
END$$


DELIMITER ;

SET SQL_MODE=@OLD_SQL_MODE;
SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS;
SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS;
