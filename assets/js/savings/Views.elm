module Savings.Views exposing (editView, renderSaving, renderSavings)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))
import Savings.Messages exposing (..)
import Savings.Models exposing (Saving)


editView : Saving -> Html Savings.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.title, onInput Savings.Messages.TitleUpdated ] []
        , input [ type_ "text", value (String.fromFloat model.amount), onInput Savings.Messages.AmountUpdated ] []
        , button [ onClick Savings.Messages.SaveSaving ] [ text "Save" ]
        , button [ onClick Savings.Messages.DeleteSaving ] [ text "Delete" ]
        ]


renderSavings : List Saving -> Html Model.Msg
renderSavings savings =
    div []
        [ button [ onClick CreateSaving ] [ text "New Saving" ]
        , table
            []
            [ thead []
                [ tr []
                    [ th [] [ text "Title" ]
                    , th [] [ text "Amount" ]
                    ]
                ]
            , tbody [] (List.map renderSaving savings)
            ]
        ]


renderSaving : Saving -> Html Model.Msg
renderSaving saving =
    tr []
        [ td [ onClick (OpenSavingEditor saving) ] [ text saving.title ]
        , td [] [ text (String.fromFloat saving.amount) ]
        ]
