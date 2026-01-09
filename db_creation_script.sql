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
  `start_date` TIMESTAMP NOT NULL,
  `end_date` TIMESTAMP NOT NULL,
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
  PRIMARY KEY (`id`),
  INDEX `fk_challenges_events1_idx` (`event_id` ASC) VISIBLE,
  CONSTRAINT `fk_challenges_events1`
    FOREIGN KEY (`event_id`)
    REFERENCES `ctfpkhk`.`events` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE)
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;


-- -----------------------------------------------------
-- Table `ctfpkhk`.`users`
-- -----------------------------------------------------
CREATE TABLE IF NOT EXISTS `ctfpkhk`.`users` (
  `id` CHAR(36) NOT NULL,
  `username` VARCHAR(40) NOT NULL,
  `email` VARCHAR(90) NOT NULL,
  `pw_hash` VARCHAR(100) NOT NULL,
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `last_active_at` TIMESTAMP NOT NULL,
  `role` VARCHAR(14) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE INDEX `username_UNIQUE` (`username` ASC) VISIBLE,
  INDEX `email_idx` (`email` ASC) INVISIBLE)
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;


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
  INDEX `fk_attachments_users1_idx` (`user_id` ASC) VISIBLE,
  INDEX `fk_attachments_events1_idx` (`event_id` ASC) VISIBLE,
  INDEX `fk_attachments_challenges1_idx` (`challenge_id` ASC) VISIBLE,
  INDEX `file_name_idx` (`file_name` ASC) VISIBLE,
  CONSTRAINT `fk_attachments_challenges1`
    FOREIGN KEY (`challenge_id`)
    REFERENCES `ctfpkhk`.`challenges` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
  CONSTRAINT `fk_attachments_events1`
    FOREIGN KEY (`event_id`)
    REFERENCES `ctfpkhk`.`events` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
  CONSTRAINT `fk_attachments_users1`
    FOREIGN KEY (`user_id`)
    REFERENCES `ctfpkhk`.`users` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE)
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;


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
  INDEX `fk_leaderboard_users1_idx` (`user_id` ASC) VISIBLE,
  INDEX `fk_leaderboard_events1_idx` (`event_id` ASC) VISIBLE,
  INDEX `fk_leaderboard_challenges1_idx` (`challenge_id` ASC) VISIBLE,
  CONSTRAINT `fk_submissions_challenges1`
    FOREIGN KEY (`challenge_id`)
    REFERENCES `ctfpkhk`.`challenges` (`id`)
    ON UPDATE CASCADE,
  CONSTRAINT `fk_submissions_events1`
    FOREIGN KEY (`event_id`)
    REFERENCES `ctfpkhk`.`events` (`id`)
    ON UPDATE CASCADE,
  CONSTRAINT `fk_submissions_users1`
    FOREIGN KEY (`user_id`)
    REFERENCES `ctfpkhk`.`users` (`id`)
    ON UPDATE CASCADE)
ENGINE = InnoDB
DEFAULT CHARACTER SET = utf8mb3;


SET SQL_MODE=@OLD_SQL_MODE;
SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS;
SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS;
