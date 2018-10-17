module Adjustments.Views exposing (editView)

import Adjustments.Messages exposing (..)
import Adjustments.Models exposing (Adjustment)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))


editView : Adjustment -> Html Adjustments.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.title, onInput Adjustments.Messages.TitleUpdated ] []
        , input [ type_ "text", value (String.fromFloat model.total), onInput Adjustments.Messages.TotalUpdated ] []
        , button [ onClick Adjustments.Messages.SaveAdjustment ] [ text "Save" ]
        , button [ onClick Adjustments.Messages.DeleteAdjustment ] [ text "Delete" ]
        ]
