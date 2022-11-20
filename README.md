# @twt_2_tg_bot
Telegram bot that zhuzh shared Twitter content

## Video and image converter

Telegram doesn't provide the ability to play Twitter videos inside the app. The bot converts them to regular MP4 files. The bot allows to download images from a tweet.

![Tweet with video link converted with bot](screenshots/video.gif)
![Tweet with images link converted with bot](screenshots/image.gif)

## Thread converter

The bot allows to unroll threads.

![Tweet with thread link converted with bot](screenshots/thread.gif)

> Only text replies are supported for now

## Text converter

Some tweets may contain line breaks or even dialogs.
![Tweet with several lines of text](screenshots/original_text.jpg)

Such tweets are barely readable in Telegram. The bot keeps original formatting.

![Tweet with several lines of text link converted with the bot](screenshots/text.gif)

> This feature is outdated because Telegram fixed their previews

## Dependencies

* [teloxide](https://github.com/teloxide/teloxide)
* [egg-mode](https://github.com/egg-mode-rs/egg-mode)